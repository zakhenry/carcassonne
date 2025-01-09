use std::cmp::PartialEq;
use crate::player::{Player, RegionIndex};
use colored::{Color, ColoredString, Colorize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use crate::connected_regions::{ConnectedRegion, PlacedTileEdge, PlacedTileRegion};
use crate::tile_definitions::{CLOISTER_IN_FIELD, RIVER_TERMINATOR};

pub const TILE_WIDTH: usize = 7;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct TileCoordinate {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BoardCoordinate {
    pub(crate) x: i8,
    pub(crate) y: i8,
}

impl BoardCoordinate {
    pub(crate) fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }


    pub(crate) fn direction_to_adjacent_coordinate(&self, other: BoardCoordinate) -> CardinalDirection {
        match (self.x - other.x, self.y - other.y) {
            (0, 1) => CardinalDirection::North,
            (-1, 0) => CardinalDirection::East,
            (0, -1) => CardinalDirection::South,
            (1, 0) => CardinalDirection::West,
            _ => panic!("Coordinates are not adjacent!")
        }
    }

    pub(crate) fn adjacent_in_direction(&self, direction: &CardinalDirection) -> BoardCoordinate {
        match direction {
            CardinalDirection::North | CardinalDirection::NorthNorthEast | CardinalDirection::NorthNorthWest => BoardCoordinate::new(self.x, self.y - 1),
            CardinalDirection::EastNorthEast | CardinalDirection::East | CardinalDirection::EastSouthEast => BoardCoordinate::new(self.x + 1, self.y),
            CardinalDirection::SouthSouthEast | CardinalDirection::South  | CardinalDirection::SouthSouthWest => BoardCoordinate::new(self.x, self.y + 1),
            CardinalDirection::WestSouthWest | CardinalDirection::West | CardinalDirection::WestNorthWest => BoardCoordinate::new(self.x - 1, self.y),
        }
    }

    pub(crate) fn adjacent_coordinates(&self) -> BTreeMap<CardinalDirection, BoardCoordinate> {
        BTreeMap::from_iter(PRIMARY_CARDINAL_DIRECTIONS.iter().map(|d|(d.clone(), self.adjacent_in_direction(d))))
    }
}

#[derive(Debug, Clone)]
pub struct TilePlacement {
    pub(crate) coordinate: BoardCoordinate,
    pub(crate) rotations: u8, // count of 90° rotations from the definition (i.e. range is 0-3 inclusive)
}

// note that the diagonal corners are intentionally omitted because carcassonne tiles do not form
// connected regions from touching corners
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum CardinalDirection {
    North,
    NorthNorthEast,
    //  NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    //  SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    //  SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,

    //  NorthWest,
    NorthNorthWest,
}

// @todo the methods here are jank or verbose, and there are almost certainly better approaches.
impl CardinalDirection {

    pub (crate) fn rotate(&self, n: usize) -> Self {
        PERIMETER_REGION_DIRECTIONS.iter().cycle().skip(n * 3 + PERIMETER_REGION_DIRECTIONS.iter().position(|d|d == self).expect("should exist")).take(1).next().expect("should have a field").clone()
    }

    pub (crate) fn adjacent(&self) -> (Self, Self) {
        match self {
            Self::North => (Self::NorthNorthWest, Self::NorthNorthEast),
            Self::NorthNorthEast => (Self::North, Self::EastNorthEast),
            Self::EastNorthEast => (Self::NorthNorthEast, Self::East),
            Self::East => (Self::EastNorthEast, Self::EastSouthEast),
            Self::EastSouthEast => (Self::East, Self::SouthSouthEast),
            Self::SouthSouthEast => (Self::EastSouthEast, Self::South),
            Self::South => (Self::SouthSouthEast, Self::SouthSouthWest),
            Self::SouthSouthWest => (Self::South, Self::WestSouthWest),
            Self::WestSouthWest => (Self::SouthSouthWest, Self::West),
            Self::West => (Self::WestSouthWest, Self::WestNorthWest),
            Self::WestNorthWest => (Self::West, Self::NorthNorthWest),
            Self::NorthNorthWest => (Self::WestNorthWest, Self::North),
        }
    }

    pub(crate) fn compass_opposite(&self) -> Self {
        match self {
            CardinalDirection::North => CardinalDirection::South,
            CardinalDirection::NorthNorthEast => CardinalDirection::SouthSouthWest,
            CardinalDirection::EastNorthEast => CardinalDirection::WestSouthWest,
            CardinalDirection::East => CardinalDirection::West,
            CardinalDirection::EastSouthEast => CardinalDirection::WestNorthWest,
            CardinalDirection::SouthSouthEast => CardinalDirection::NorthNorthWest,
            CardinalDirection::South => CardinalDirection::North,
            CardinalDirection::SouthSouthWest => CardinalDirection::NorthNorthEast,
            CardinalDirection::WestSouthWest => CardinalDirection::EastNorthEast,
            CardinalDirection::West => CardinalDirection::East,
            CardinalDirection::WestNorthWest => CardinalDirection::EastSouthEast,
            CardinalDirection::NorthNorthWest => CardinalDirection::SouthSouthEast,
        }
    }

    pub(crate) fn tile_opposite(&self) -> Self {
        match self {
            CardinalDirection::North => CardinalDirection::South,
            CardinalDirection::NorthNorthEast => CardinalDirection::SouthSouthEast,
            CardinalDirection::EastNorthEast => CardinalDirection::WestNorthWest,
            CardinalDirection::East => CardinalDirection::West,
            CardinalDirection::EastSouthEast => CardinalDirection::WestSouthWest,
            CardinalDirection::SouthSouthEast => CardinalDirection::NorthNorthEast,
            CardinalDirection::South => CardinalDirection::North,
            CardinalDirection::SouthSouthWest => CardinalDirection::NorthNorthWest,
            CardinalDirection::WestSouthWest => CardinalDirection::EastSouthEast,
            CardinalDirection::West => CardinalDirection::East,
            CardinalDirection::WestNorthWest => CardinalDirection::EastNorthEast,
            CardinalDirection::NorthNorthWest => CardinalDirection::SouthSouthWest,
        }
    }

    pub(crate) fn primary_direction(&self) -> Self {
        match self {
            CardinalDirection::North => CardinalDirection::North,
            CardinalDirection::NorthNorthEast => CardinalDirection::North,
            CardinalDirection::EastNorthEast => CardinalDirection::East,
            CardinalDirection::East => CardinalDirection::East,
            CardinalDirection::EastSouthEast => CardinalDirection::East,
            CardinalDirection::SouthSouthEast => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::South,
            CardinalDirection::SouthSouthWest => CardinalDirection::South,
            CardinalDirection::WestSouthWest => CardinalDirection::West,
            CardinalDirection::West => CardinalDirection::West,
            CardinalDirection::WestNorthWest => CardinalDirection::West,
            CardinalDirection::NorthNorthWest => CardinalDirection::North,
        }
    }
}

pub(crate) const PRIMARY_CARDINAL_DIRECTIONS: &[CardinalDirection; 4] = &[
    CardinalDirection::North,
    CardinalDirection::East,
    CardinalDirection::South,
    CardinalDirection::West,
];

pub(crate) const PERIMETER_REGION_DIRECTIONS: &[CardinalDirection; 12] = &[
    CardinalDirection::NorthNorthWest,
    CardinalDirection::North,
    CardinalDirection::NorthNorthEast,
    CardinalDirection::EastNorthEast,
    CardinalDirection::East,
    CardinalDirection::EastSouthEast,
    CardinalDirection::SouthSouthEast,
    CardinalDirection::South,
    CardinalDirection::SouthSouthWest,
    CardinalDirection::WestSouthWest,
    CardinalDirection::West,
    CardinalDirection::WestNorthWest,
];

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) enum RegionType {
    City,
    Field,
    Cloister,
    Road,
    Water,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Region {
    City {
        edges: &'static [CardinalDirection],
        meeple_coordinate: TileCoordinate,
        pennant: bool,
    },
    Field {
        edges: &'static [CardinalDirection],
        meeple_coordinate: TileCoordinate,
    },
    Cloister {
        meeple_coordinate: TileCoordinate,
    },
    Road {
        edges: &'static [CardinalDirection],
        meeple_coordinate: TileCoordinate,
    },
    Water {
        edges: &'static [CardinalDirection],
    },
}

impl Region {
    pub(crate) fn edges(&self) -> &'static [CardinalDirection] {
        match self {
            Region::City { edges, .. } => edges,
            Region::Field { edges, .. } => edges,
            Region::Road { edges, .. } => edges,
            Region::Water { edges, .. } => edges,
            Region::Cloister { .. } => &[],
        }
    }

    // @todo the existence of the two enums is a code smell. Some refactoring is needed!
    pub(crate) fn region_type(&self) -> RegionType {
        match self {
            Region::City { .. } => RegionType::City,
            Region::Field { .. } => RegionType::Field,
            Region::Road { .. } => RegionType::Road,
            Region::Water { .. } => RegionType::Water,
            Region::Cloister { .. } => RegionType::Cloister,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expansion {
    River,
}

#[derive(Debug, PartialEq)]
pub struct TileDefinition {
    pub(crate) count: u8,
    pub(crate) name: &'static str,
    pub(crate) render: TileRenderRepresentation,
    pub(crate) regions: &'static [Region],
    pub(crate) expansion: Option<Expansion>,
}

impl TileDefinition {
    // @todo consider BTreeMap to give an order to the map, thus eliminating `perimeter_regions` method
    fn directed_regions(&self) -> HashMap<CardinalDirection, &Region> {
        let mut map = HashMap::with_capacity(PERIMETER_REGION_DIRECTIONS.len());

        for region in self.regions {
            for edge in region.edges() {
                map.insert(*edge, region);
            }
        }

        map
    }
    /// The list of region types around the perimeter of the definition
    /// * no rotation applied
    /// * starting from NorthNorthWest, going clockwise
    fn perimeter_regions(&self) -> Vec<RegionType> {
        let directed_regions = self.directed_regions();

        PERIMETER_REGION_DIRECTIONS
            .iter()
            .filter_map(|direction| directed_regions.get(direction).map(|r| r.region_type()))
            .collect()
    }

    pub(crate) fn list_oriented_region_types(&self, rotations: u8) -> Vec<RegionType> {
        let perimeter = self.perimeter_regions();

        perimeter
            .into_iter()
            .cycle()
            .skip(((4 - rotations) * 3) as usize)
            .take(12)
            .collect()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum RenderCell {
    Field,
    Road,
    City,
    Cloister,
    Pennant,
    Water,
    Corner, // @todo remove?
}

impl RenderCell {
    fn colors(&self) -> (Color, Color, Color, Color) {
        match self {
            Self::Field => (
                Color::Green,
                Color::TrueColor {
                    r: 143,
                    g: 185,
                    b: 45,
                },
                Color::TrueColor {
                    r: 165,
                    g: 184,
                    b: 90,
                },
                Color::TrueColor {
                    r: 85,
                    g: 122,
                    b: 30,
                },
            ),
            Self::Road => (
                Color::BrightBlack,
                Color::TrueColor {
                    r: 190,
                    g: 190,
                    b: 190,
                },
                Color::TrueColor {
                    r: 220,
                    g: 220,
                    b: 220,
                },
                Color::TrueColor {
                    r: 150,
                    g: 150,
                    b: 150,
                },
            ),
            Self::City => (
                Color::Yellow,
                Color::TrueColor {
                    r: 199,
                    g: 147,
                    b: 88,
                },
                Color::TrueColor {
                    r: 208,
                    g: 169,
                    b: 116,
                },
                Color::TrueColor {
                    r: 154,
                    g: 94,
                    b: 56,
                },
            ),
            Self::Cloister => (
                Color::BrightWhite,
                Color::BrightWhite,
                Color::BrightWhite,
                Color::BrightWhite,
            ),
            Self::Pennant => (Color::Red, Color::Red, Color::Red, Color::Red),
            Self::Water => (
                Color::Blue,
                Color::TrueColor {
                    r: 143,
                    g: 163,
                    b: 215,
                },
                Color::TrueColor {
                    r: 173,
                    g: 186,
                    b: 221,
                },
                Color::TrueColor {
                    r: 123,
                    g: 142,
                    b: 177,
                },
            ),
            Self::Corner => (Color::Black, Color::Red, Color::Red, Color::Red),
        }
    }
}

pub enum RenderStyle {
    Ansi,
    TrueColor,
    // image??
}

#[derive(Debug, Clone)]
pub struct PlacedTile {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) placement: TilePlacement,
}

impl PlacedTile {

    pub(crate) fn new(tile: &'static TileDefinition, x: i8, y: i8, rotations: u8) -> Self {
        PlacedTile { tile, placement: TilePlacement { coordinate: BoardCoordinate { x, y }, rotations } }
    }

    pub(crate) fn get_opposite_river_end_direction(&self, direction: CardinalDirection) -> Option<CardinalDirection> {

        if self.tile == &RIVER_TERMINATOR {
            return None
        }

        let region = self.tile.regions.iter().filter(|r|matches!(r, Region::Water {..})).next().expect("should be river tile");

        let rotated_edges: Vec<_> = region.edges().iter().map(|d|d.rotate(self.placement.rotations as usize)).collect();
        assert_eq!(rotated_edges.len(), 2);

        rotated_edges.into_iter().filter(|&e|e != direction).next()
    }

    pub(crate) fn list_regions_on_edge(
        &self,
        cardinal_direction: &CardinalDirection,
    ) -> Vec<RegionType> {
        let edges = self.tile.list_oriented_region_types(self.placement.rotations);

        let skip = match cardinal_direction {
            CardinalDirection::North => 0,
            CardinalDirection::East => 3,
            CardinalDirection::South => 6,
            CardinalDirection::West => 9,
            _ => panic!("only primary cardinal directions are supported"),
        };

        edges.into_iter().skip(skip).take(3).collect()
    }

    pub(crate) fn list_placed_tile_regions(&self) -> Vec<PlacedTileRegion> {

        (0..self.tile.regions.len()).map(|idx|{
            PlacedTileRegion::new(self, RegionIndex::new(idx))
        }).collect()

    }

    pub fn render_to_lines(
        &self, /* @todo placed meeple */
        render_style: RenderStyle,
    ) -> Vec<String> {
        self.tile
            .render
            .rotated(self.placement.rotations)
            .enumerate()
            .map(|(row_idx, row)| {
                let chars: String = row
                    .enumerate()
                    .map(|(column_idx, cell)| {
                        // @todo consider performance of this call

                        let (portable, primary_true, lighten_true, darken_true) = cell.colors();

                        match render_style {
                            RenderStyle::Ansi => if let RenderCell::Corner = cell {
                                " ".black()
                            } else if row_idx == 0 {
                                "━━".black().on_color(portable)
                            } else if column_idx == 0 {
                                "┃".black().on_color(portable)
                            } else if row_idx == TILE_WIDTH - 1 {
                                "━━".black().on_color(portable)
                            } else if column_idx == TILE_WIDTH - 1 {
                                "┃".black().on_color(portable)
                            } else {
                                "  ".on_color(portable)
                            }
                            .to_string(),
                            RenderStyle::TrueColor => {
                                if let RenderCell::Corner = cell {
                                    "  ".to_string()
                                } else if row_idx == 0 {
                                    "▄▄".color(primary_true).on_color(lighten_true).to_string()
                                } else if column_idx == 0 {
                                    " ".on_color(lighten_true).to_string()
                                        + &" ".on_color(primary_true).to_string()
                                } else if row_idx == TILE_WIDTH - 1 {
                                    "▄▄".color(darken_true).on_color(primary_true).to_string()
                                } else if column_idx == TILE_WIDTH - 1 {
                                    " ".on_color(primary_true).to_string()
                                        + &" ".on_color(darken_true).to_string()
                                } else {
                                    match cell {
                                        RenderCell::Pennant => " ⛨".bold().color(Color::TrueColor {
                                            r: 0,
                                            g: 100,
                                            b: 174,
                                        }).on_color(Color::TrueColor {
                                            r: 199,
                                            g: 147,
                                            b: 88,
                                        }).to_string(),
                                        _ => "  ".on_color(primary_true).to_string()
                                    }
                                }
                            }
                        }
                    })
                    .collect();

                chars
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct TileRenderRepresentation(pub [[RenderCell; 7]; 7]);

impl TileRenderRepresentation {
    pub(crate) fn rotated<'a>(
        &'a self,
        rotations: u8,
    ) -> impl Iterator<Item = impl Iterator<Item = &'a RenderCell>> + 'a {
        let rotations = rotations % 4; // Normalize rotations to 0..3

        (0..TILE_WIDTH).map(move |r| {
            (0..TILE_WIDTH).map(move |c| match rotations {
                1 => &self.0[TILE_WIDTH - c - 1][r], // 90 degrees clockwise
                2 => &self.0[TILE_WIDTH - r - 1][TILE_WIDTH - c - 1], // 180 degrees
                3 => &self.0[c][TILE_WIDTH - r - 1], // 270 degrees clockwise
                _ => &self.0[r][c],                  // No rotation
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RegionType::*;
    use super::*;
    use crate::tile_definitions::{CORNER_ROAD, SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE};

    #[test]
    fn test_perimeter_regions_returns_expected_result() {
        let perimeter = SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE.perimeter_regions();

        assert_eq!(
            perimeter,
            vec![City, City, City, Field, Water, Field, Field, Road, Field, Field, Water, Field]
        )
    }
    #[test]
    fn test_oriented_regions_returns_expected_result() {
        let perimeter = SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE.list_oriented_region_types(3);

        assert_eq!(
            perimeter,
            vec![Field, Water, Field, City, City, City, Field, Water, Field, Field, Road, Field]
        )
    }

    #[test]
    fn test_list_regions_on_edge() {
        let tile = PlacedTile {
            tile: &SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE,
            placement: TilePlacement {
                rotations: 0,
                coordinate: BoardCoordinate::new(0, 0),
            },
        };

        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::North),
            vec![City, City, City]
        );
        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::East),
            vec![Field, Water, Field]
        );
        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::South),
            vec![Field, Road, Field]
        );
        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::West),
            vec![Field, Water, Field]
        );
    }

    #[test]
    fn test_get_regions_on_edge_rotated() {
        let tile = PlacedTile {
            tile: &CORNER_ROAD,
            placement: TilePlacement {
                rotations: 1,
                coordinate: BoardCoordinate::new(0, 0),
            },
        };

        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::South),
            vec![Field, Road, Field]
        );
    }

    #[test]
    fn test_direction_to_adjacent_coordinate() {
        assert_eq!(BoardCoordinate{x: 0, y: 0}.direction_to_adjacent_coordinate(BoardCoordinate {x: 0, y: -1}), CardinalDirection::North);
        assert_eq!(BoardCoordinate{x: 0, y: 0}.direction_to_adjacent_coordinate(BoardCoordinate {x: 1, y: 0}), CardinalDirection::East);
        assert_eq!(BoardCoordinate{x: 0, y: 0}.direction_to_adjacent_coordinate(BoardCoordinate {x: 0, y: 1}), CardinalDirection::South);
        assert_eq!(BoardCoordinate{x: 0, y: 0}.direction_to_adjacent_coordinate(BoardCoordinate {x: -1, y: 0}), CardinalDirection::West);
    }

    #[test]
    #[should_panic]
    fn test_direction_to_adjacent_coordinate_panics_on_non_adjacent() {
        BoardCoordinate{x: 0, y: 0}.direction_to_adjacent_coordinate(BoardCoordinate {x: 1, y: 1});
    }


    #[test]
    fn test_rotate_cardinal_direction() {
        assert_eq!(CardinalDirection::North.rotate(0), CardinalDirection::North);
        assert_eq!(CardinalDirection::North.rotate(1), CardinalDirection::East);
        assert_eq!(CardinalDirection::NorthNorthWest.rotate(2), CardinalDirection::NorthNorthWest.compass_opposite());
    }

    #[test]
    fn test_adjacent_cardinal_direction() {
        assert_eq!(CardinalDirection::North.adjacent(), (CardinalDirection::NorthNorthWest, CardinalDirection::NorthNorthEast));
        assert_eq!(CardinalDirection::EastSouthEast.adjacent(), (CardinalDirection::East, CardinalDirection::SouthSouthEast));
    }

}
