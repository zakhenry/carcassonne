use std::collections::HashMap;
use std::fmt::Debug;
use colored::{Color, ColoredString, Colorize};
use crate::player::Player;

pub const TILE_WIDTH: usize = 7;

#[derive(Debug, PartialEq)]
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
        Self {x, y}
    }

    pub(crate) fn adjacent_coordinates(&self) -> HashMap<CardinalDirection, BoardCoordinate> {
        HashMap::from([
            (CardinalDirection::North, BoardCoordinate::new(self.x, self.y - 1)),
            (CardinalDirection::East, BoardCoordinate::new(self.x + 1, self.y)),
            (CardinalDirection::South, BoardCoordinate::new(self.x, self.y + 1)),
            (CardinalDirection::West, BoardCoordinate::new(self.x - 1, self.y)),
        ])
    }
}

#[derive(Debug)]
pub struct TilePlacement {
    pub(crate) coordinate: BoardCoordinate,
    pub(crate) rotations: u8, // count of 90° rotations from the definition (i.e. range is 0-3 inclusive)
}

// note that the diagonal corners are intentionally omitted because carcassonne tiles do not form
// connected regions from touching corners
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

pub(crate) const PRIMARY_CARDINAL_DIRECTIONS: &[CardinalDirection; 4] = &[
    CardinalDirection::North,
    CardinalDirection::East,
    CardinalDirection::South,
    CardinalDirection::West
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum RegionType {
    City,
    Field,
    Cloister,
    Road,
    Water
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
    fn edges(&self) -> &'static [CardinalDirection] {
        match self {
            Region::City { edges, .. } => edges,
            Region::Field { edges, .. } => edges,
            Region::Road { edges, .. } => edges,
            Region::Water { edges, .. } => edges,
            Region::Cloister { .. } => &[]
        }
    }

    // @todo the existence of the two enums is a code smell. Some refactoring is needed!
    fn region_type(&self) -> RegionType {
        match self {
            Region::City { .. } => RegionType::City,
            Region::Field { .. } => RegionType::Field,
            Region::Road { .. } => RegionType::Road,
            Region::Water { .. } => RegionType::Water,
            Region::Cloister { .. } => RegionType::Cloister
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expansion {
    River
}

#[derive(Debug, PartialEq)]
pub struct TileDefinition {
    pub(crate) count: u8,
    pub(crate) name: &'static str,
    pub(crate) render: TileRenderRepresentation,
    pub(crate) regions: &'static [Region],
    pub(crate) expansion: Option<Expansion>
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

        PERIMETER_REGION_DIRECTIONS.iter().filter_map(|direction|directed_regions.get(direction).map(|r|r.region_type())).collect()
    }

    pub(crate) fn list_oriented_regions(&self, rotations: u8) -> Vec<RegionType> {
        let perimeter = self.perimeter_regions().into_iter().cycle();

        perimeter.skip((rotations * 3) as usize).take(12).collect()
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
            Self::Field => (Color::Green, Color::TrueColor { r: 143, g: 185, b: 45}, Color::TrueColor { r: 165, g: 184, b: 90}, Color::TrueColor { r: 85, g: 122, b: 30}),
            Self::Road => (Color::BrightBlack, Color::TrueColor { r: 190, g: 190, b: 190}, Color::TrueColor { r: 220, g: 220, b: 220}, Color::TrueColor { r: 150, g: 150, b: 150}),
            Self::City => (Color::Yellow, Color::TrueColor { r: 199, g: 147, b: 88}, Color::TrueColor { r: 208, g: 169, b: 116}, Color::TrueColor { r: 154, g: 94, b: 56}),
            Self::Cloister => (Color::BrightWhite, Color::BrightWhite, Color::BrightWhite, Color::BrightWhite),
            Self::Pennant => (Color::Red, Color::Red, Color::Red, Color::Red),
            Self::Water => (Color::Blue, Color::TrueColor { r: 143, g: 163, b: 215}, Color::TrueColor { r: 173, g: 186, b: 221}, Color::TrueColor { r: 123, g: 142, b: 177}),
            Self::Corner => (Color::Black, Color::Red, Color::Red, Color::Red),
        }
    }

}

pub enum RenderStyle {
    Ansi,
    TrueColor,
    // image??
}

#[derive(Debug)]
pub struct PlacedTile {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) placement: TilePlacement,
}

impl PlacedTile {

    pub fn render_to_lines(&self /* @todo placed meeple */, render_style: RenderStyle) -> Vec<String> {
        self.tile.render.rotated(self.placement.rotations).enumerate().map(|(row_idx, row)| {
            let chars: String = row.enumerate().map(|(column_idx, cell)| {

                // @todo consider performance of this call

                let (portable, primary_true, lighten_true, darken_true) = cell.colors();

                match render_style {
                    RenderStyle::Ansi => {
                        if let RenderCell::Corner = cell {
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
                        }.to_string()
                    }
                    RenderStyle::TrueColor => {


                        if let RenderCell::Corner = cell {
                            "  ".to_string()
                        } else if row_idx == 0 {
                            "▄▄".color(primary_true).on_color(lighten_true).to_string()
                        } else if column_idx == 0 {
                            " ".on_color(lighten_true).to_string() + &" ".on_color(primary_true).to_string()
                        } else if row_idx == TILE_WIDTH - 1 {
                            "▄▄".color(darken_true).on_color(primary_true).to_string()
                        } else if column_idx == TILE_WIDTH - 1 {
                            " ".on_color(primary_true).to_string() + &" ".on_color(darken_true).to_string()
                        } else {
                            "  ".on_color(primary_true).to_string()
                        }
                    }
                }


            }).collect();

            chars
        }).collect()
    }

}




#[derive(Debug, PartialEq)]
pub(crate) struct TileRenderRepresentation(pub [[RenderCell; 7]; 7]);

impl TileRenderRepresentation {
    pub(crate) fn rotated<'a>(&'a self, rotations: u8) -> impl Iterator<Item = impl Iterator<Item = &'a RenderCell>> + 'a {

        let rotations = rotations % 4; // Normalize rotations to 0..3

        (0..TILE_WIDTH).map(move |r| {
            (0..TILE_WIDTH).map(move |c| match rotations {
                1 => &self.0[TILE_WIDTH - c - 1][r], // 90 degrees clockwise
                2 => &self.0[TILE_WIDTH - r - 1][TILE_WIDTH - c - 1], // 180 degrees
                3 => &self.0[c][TILE_WIDTH - r - 1], // 270 degrees clockwise
                _ => &self.0[r][c], // No rotation
            })
        })

    }
}


#[cfg(test)]
mod tests {
    use crate::tile_definitions::SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE;
    use super::*;
    use super::RegionType::*;

    #[test]
    fn test_perimeter_regions_returns_expected_result() {

        let perimeter = SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE.perimeter_regions();

        assert_eq!(perimeter, vec![City, City, City, Field, Water, Field, Field, Road, Field, Field, Water, Field])

    }
    #[test]
    fn test_oriented_regions_returns_expected_result() {

        let perimeter = SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE.list_oriented_regions(3);

        assert_eq!(perimeter, vec![Field, Water, Field, City, City, City, Field, Water, Field, Field, Road, Field])

    }

}
