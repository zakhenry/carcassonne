use crate::connected_regions::{
    ConnectedRegion, ConnectedRegionId, PlacedTileEdge, PlacedTileRegion,
};
use crate::player::{Meeple, MeepleColor, RegionIndex};
use crate::tile_definitions::RIVER_TERMINATOR;
use colored::{Color, Colorize};
use std::cmp::PartialEq;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use uuid::Uuid;

pub const TILE_WIDTH: usize = 7;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct TileCoordinate {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl TileCoordinate {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub(crate) fn rotate_around_center(&self, rotations: u8) -> Self {
        let rotations = rotations % 4;
        match rotations {
            0 => self.clone(),                                                // No rotation
            1 => Self::new(TILE_WIDTH - 1 - self.y, self.x), // 90 degrees clockwise
            2 => Self::new(TILE_WIDTH - 1 - self.x, TILE_WIDTH - 1 - self.y), // 180 degrees
            3 => Self::new(self.y, TILE_WIDTH - 1 - self.x), // 270 degrees clockwise
            _ => unreachable!(),                             // This branch is logically unreachable
        }
    }
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

    pub(crate) fn direction_to_adjacent_coordinate(
        &self,
        other: BoardCoordinate,
    ) -> CardinalDirection {
        match (self.x - other.x, self.y - other.y) {
            (0, 1) => CardinalDirection::North,
            (-1, 0) => CardinalDirection::East,
            (0, -1) => CardinalDirection::South,
            (1, 0) => CardinalDirection::West,
            _ => panic!("Coordinates are not adjacent!"),
        }
    }

    pub(crate) fn adjacent_in_direction(&self, direction: &CardinalDirection) -> BoardCoordinate {
        match direction {
            CardinalDirection::North
            | CardinalDirection::NorthNorthEast
            | CardinalDirection::NorthNorthWest => BoardCoordinate::new(self.x, self.y - 1),
            CardinalDirection::EastNorthEast
            | CardinalDirection::East
            | CardinalDirection::EastSouthEast => BoardCoordinate::new(self.x + 1, self.y),
            CardinalDirection::SouthSouthEast
            | CardinalDirection::South
            | CardinalDirection::SouthSouthWest => BoardCoordinate::new(self.x, self.y + 1),
            CardinalDirection::WestSouthWest
            | CardinalDirection::West
            | CardinalDirection::WestNorthWest => BoardCoordinate::new(self.x - 1, self.y),
        }
    }

    pub(crate) fn adjacent_coordinates(&self) -> BTreeMap<CardinalDirection, BoardCoordinate> {
        BTreeMap::from_iter(
            PRIMARY_CARDINAL_DIRECTIONS
                .iter()
                .map(|d| (*d, self.adjacent_in_direction(d))),
        )
    }

    pub(crate) fn surrounding_coordinates(&self) -> Vec<BoardCoordinate> {
        (-1..=1).flat_map(|x|{
            (-1..=1).filter_map(move |y|{
                if x == 0 && y == 0 {
                    None
                } else {
                    Some(BoardCoordinate { x: self.x + x, y: self.y + y })
                }
            })
        }).collect()
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
    pub(crate) fn rotate(&self, n: usize) -> Self {
        *PERIMETER_REGION_DIRECTIONS
            .iter()
            .cycle()
            .skip(
                n * 3
                    + PERIMETER_REGION_DIRECTIONS
                        .iter()
                        .position(|d| d == self)
                        .expect("should exist"),
            )
            .take(1)
            .next()
            .expect("should have a field")
    }

    pub(crate) fn adjacent(&self) -> (Self, Self) {
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

    pub(crate) fn meeple_coordinate_rotated(&self, rotations: u8) -> Option<TileCoordinate> {
        let coordinate = match self {
            Region::City {
                meeple_coordinate, ..
            } => Some(meeple_coordinate),
            Region::Field {
                meeple_coordinate, ..
            } => Some(meeple_coordinate),
            Region::Road {
                meeple_coordinate, ..
            } => Some(meeple_coordinate),
            Region::Water { .. } => None,
            Region::Cloister {
                meeple_coordinate, ..
            } => Some(meeple_coordinate),
        }
        .cloned();

        coordinate.map(|coord| coord.rotate_around_center(rotations))
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

impl MeepleColor {
    fn render_color(&self, style: &RenderStyle) -> Color {
        match (self, style) {
            (MeepleColor::Red, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Red,
            (MeepleColor::Green, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Green,
            (MeepleColor::Blue, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Blue,
            (MeepleColor::Black, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Black,

            (MeepleColor::Red, RenderStyle::TrueColor) => Color::TrueColor {
                r: 194,
                g: 0,
                b: 25,
            },
            (MeepleColor::Green, RenderStyle::TrueColor) => Color::TrueColor {
                r: 16,
                g: 126,
                b: 50,
            },
            (MeepleColor::Blue, RenderStyle::TrueColor) => Color::TrueColor {
                r: 10,
                g: 79,
                b: 147,
            },
            (MeepleColor::Black, RenderStyle::TrueColor) => Color::TrueColor {
                r: 43,
                g: 42,
                b: 44,
            },
        }
    }
}

impl RenderCell {
    fn ascii_code(&self) -> &'static str {
        match self {
            RenderCell::Field => "░░",
            RenderCell::Road => "██",
            RenderCell::City => "▓▓",
            RenderCell::Cloister => " ✝",
            RenderCell::Pennant => " ⛨",
            RenderCell::Water => "~~",
            RenderCell::Corner => " ",
        }
    }

    fn render_ascii(&self, row_idx: usize, column_idx: usize, meeple: Option<&Meeple>) -> String {
        if let RenderCell::Corner = self {
            " "
        } else if row_idx == 0 {
            "━━"
        } else if column_idx == 0 {
            "┃"
        } else if row_idx == TILE_WIDTH - 1 {
            "━━"
        } else if column_idx == TILE_WIDTH - 1 {
            "┃"
        } else {
            if meeple.is_some() {
                "ꆜ "
            } else {
                self.ascii_code()
            }
        }
        .to_string()
    }

    fn render_ansi(&self, row_idx: usize, column_idx: usize, meeple: Option<&Meeple>) -> String {
        let color = match self {
            Self::Field => Color::Green,
            Self::Road => Color::BrightBlack,
            Self::City => Color::Yellow,
            Self::Cloister => Color::BrightWhite,
            Self::Pennant => Color::Red,
            Self::Water => Color::Blue,
            Self::Corner => Color::Black,
        };

        if let RenderCell::Corner = self {
            " ".black()
        } else if row_idx == 0 {
            "━━".black().on_color(color)
        } else if column_idx == 0 {
            "┃".black().on_color(color)
        } else if row_idx == TILE_WIDTH - 1 {
            "━━".black().on_color(color)
        } else if column_idx == TILE_WIDTH - 1 {
            "┃".black().on_color(color)
        } else {
            "  ".on_color(color)
        }
        .to_string()
    }

    fn render_true_color(
        &self,
        row_idx: usize,
        column_idx: usize,
        meeple: Option<&Meeple>,
    ) -> String {
        let (primary, light, dark) = match self {
            Self::Field => (
                Color::TrueColor {
                    r: 143,
                    g: 185,
                    b: 45,
                },
                Color::TrueColor {
                    r: 153,
                    g: 195,
                    b: 55,
                },
                Color::TrueColor {
                    r: 135,
                    g: 175,
                    b: 35,
                },
            ),
            Self::Road => (
                Color::TrueColor {
                    r: 190,
                    g: 190,
                    b: 190,
                },
                Color::TrueColor {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                Color::TrueColor {
                    r: 180,
                    g: 180,
                    b: 180,
                },
            ),
            Self::City => (
                Color::TrueColor {
                    r: 199,
                    g: 147,
                    b: 88,
                },
                Color::TrueColor {
                    r: 209,
                    g: 157,
                    b: 98,
                },
                Color::TrueColor {
                    r: 189,
                    g: 137,
                    b: 78,
                },
            ),
            Self::Cloister => (Color::BrightWhite, Color::BrightWhite, Color::BrightWhite),
            Self::Pennant => (Color::Red, Color::Red, Color::Red),
            Self::Water => (
                Color::TrueColor {
                    r: 143,
                    g: 163,
                    b: 215,
                },
                Color::TrueColor {
                    r: 153,
                    g: 173,
                    b: 225,
                },
                Color::TrueColor {
                    r: 133,
                    g: 153,
                    b: 205,
                },
            ),
            Self::Corner => (Color::Red, Color::Red, Color::Red),
        };

        if let RenderCell::Corner = self {
            "  ".to_string()
        } else if row_idx == 0 {
            "▄▄".color(primary).on_color(light).to_string()
        } else if column_idx == 0 {
            " ".on_color(light).to_string() + &" ".on_color(primary).to_string()
        } else if row_idx == TILE_WIDTH - 1 {
            "▄▄".color(dark).on_color(primary).to_string()
        } else if column_idx == TILE_WIDTH - 1 {
            " ".on_color(primary).to_string() + &" ".on_color(dark).to_string()
        } else {
            match self {
                RenderCell::Pennant => " ⛨"
                    .bold()
                    .color(Color::TrueColor {
                        r: 0,
                        g: 100,
                        b: 174,
                    })
                    .on_color(Color::TrueColor {
                        r: 199,
                        g: 147,
                        b: 88,
                    })
                    .to_string(),
                _ => {
                    if let Some(meeple) = meeple {
                        "ꆜ"
                            .bold()
                            .color(meeple.color.render_color(&RenderStyle::TrueColor))
                            .on_color(primary)
                            .to_string()
                    } else {
                        "  ".on_color(primary).to_string()
                    }
                }
            }
        }
    }
}

pub enum RenderStyle {
    Ansi,
    TrueColor,
    Ascii,
    // image??
}

#[derive(Debug, Clone)]
pub struct PlacedTile {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) placement: TilePlacement,
    pub(crate) meeple: Option<(RegionIndex, Meeple)>,
}

impl PlacedTile {
    pub(crate) fn new(tile: &'static TileDefinition, x: i8, y: i8, rotations: u8) -> Self {
        PlacedTile {
            tile,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x, y },
                rotations,
            },
            meeple: Default::default(),
        }
    }

    pub(crate) fn new_with_meeple(tile: &'static TileDefinition, x: i8, y: i8, rotations: u8, meeple_placement: (RegionIndex, Meeple)) -> Self {
        let mut tile = Self::new(tile, x, y, rotations);

        tile.meeple = Some(meeple_placement);

        tile
    }

    pub(crate) fn has_occupied_cloister(&self) -> bool {

        if let Some((meeple_index, _)) = self.meeple {
            if let Some(cloister_index) = self.tile.regions.iter().enumerate().find_map(|(index, r)|if let Region::Cloister {..} = r { Some(index)} else {None}) {
                cloister_index == *meeple_index
            } else {
                false
            }

        } else {
            false
        }
    }

    pub(crate) fn own_connected_regions(&self) -> Vec<ConnectedRegion> {
        let regions = self.list_placed_tile_regions();

        let mut connected_regions: Vec<_> = regions
            .into_iter()
            .map(|region| {
                let connected_edges: HashMap<PlacedTileEdge, Option<PlacedTileEdge>> = region
                    .region
                    .edges()
                    .iter()
                    .map(|edge| {
                        (
                            PlacedTileEdge {
                                global_direction: edge.rotate(self.placement.rotations as usize),
                                coordinate: self.placement.coordinate,
                            },
                            None,
                        )
                    })
                    .collect();

                ConnectedRegion {
                    region_type: region.region.region_type(),
                    tile_regions: Vec::from([region]),
                    id: Uuid::new_v4(),
                    adjacent_regions: Default::default(),
                    connected_edges,
                }
            })
            .collect();

        let edge_index: HashMap<CardinalDirection, ConnectedRegionId> = connected_regions
            .iter()
            .flat_map(|connected_region| {
                connected_region.tile_regions.iter().flat_map(|region| {
                    region
                        .region
                        .edges()
                        .iter()
                        .map(|d| (*d, connected_region.id))
                })
            })
            .collect();

        for connected_region in connected_regions.iter_mut() {
            for edge in connected_region
                .tile_regions
                .first()
                .expect("should have one")
                .region
                .edges()
            {
                let (left, right) = edge.adjacent();
                for adjacent in [left, right] {
                    let connected_region_id = *edge_index
                        .get(&adjacent)
                        .expect("all edges should be indexed");
                    if connected_region_id != connected_region.id {
                        connected_region
                            .adjacent_regions
                            .insert(connected_region_id);
                    }
                }
            }
        }

        connected_regions
    }

    pub(crate) fn get_opposite_river_end_direction(
        &self,
        direction: CardinalDirection,
    ) -> Option<CardinalDirection> {
        if self.tile == &RIVER_TERMINATOR {
            return None;
        }

        let region = self
            .tile
            .regions
            .iter()
            .find(|r| matches!(r, Region::Water { .. }))
            .expect("should be river tile");

        let rotated_edges: Vec<_> = region
            .edges()
            .iter()
            .map(|d| d.rotate(self.placement.rotations as usize))
            .collect();
        assert_eq!(rotated_edges.len(), 2);

        rotated_edges.into_iter().find(|&e| e != direction)
    }

    pub(crate) fn list_regions_on_edge(
        &self,
        cardinal_direction: &CardinalDirection,
    ) -> Vec<RegionType> {
        let edges = self
            .tile
            .list_oriented_region_types(self.placement.rotations);

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
        (0..self.tile.regions.len())
            .map(|idx| PlacedTileRegion::new(self, RegionIndex::new(idx)))
            .collect()
    }

    pub fn render_to_lines(&self, render_style: &RenderStyle) -> Vec<String> {
        let meeple_render_coordinate = if let Some((meeple_region_index, meeple)) = &self.meeple {
            self.tile
                .regions
                .iter()
                .enumerate()
                .find_map(|(tile_region_index, region)| {
                    if **meeple_region_index == tile_region_index {
                        region.meeple_coordinate_rotated(self.placement.rotations)
                    } else {
                        None
                    }
                    .map(|coordinate| (coordinate, meeple))
                })
        } else {
            None
        };

        self.tile
            .render
            .rotated(self.placement.rotations)
            .enumerate()
            .map(|(row_idx, row)| {
                let chars: String = row
                    .enumerate()
                    .map(|(column_idx, cell): (usize, &RenderCell)| {
                        let tile_coord = TileCoordinate::new(column_idx, row_idx);

                        let meeple = match &meeple_render_coordinate {
                            Some((coordinate, meeple)) if coordinate == &tile_coord => Some(meeple),
                            _ => None,
                        };

                        match render_style {
                            RenderStyle::Ascii => {
                                cell.render_ascii(row_idx, column_idx, meeple.map(|v| &**v))
                            }
                            RenderStyle::Ansi => {
                                cell.render_ansi(row_idx, column_idx, meeple.map(|v| &**v))
                            }
                            RenderStyle::TrueColor => {
                                cell.render_true_color(row_idx, column_idx, meeple.map(|v| &**v))
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
    pub(crate) fn rotated(
        &self,
        rotations: u8,
    ) -> impl Iterator<Item = impl Iterator<Item = &RenderCell>> {
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
            vec![Field, Water, Field, Field, Road, Field, Field, Water, Field, City, City, City]
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
            meeple: Default::default(),
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
            meeple: Default::default(),
        };

        assert_eq!(
            tile.list_regions_on_edge(&CardinalDirection::South),
            vec![Field, Road, Field]
        );
    }

    #[test]
    fn test_direction_to_adjacent_coordinate() {
        assert_eq!(
            BoardCoordinate { x: 0, y: 0 }
                .direction_to_adjacent_coordinate(BoardCoordinate { x: 0, y: -1 }),
            CardinalDirection::North
        );
        assert_eq!(
            BoardCoordinate { x: 0, y: 0 }
                .direction_to_adjacent_coordinate(BoardCoordinate { x: 1, y: 0 }),
            CardinalDirection::East
        );
        assert_eq!(
            BoardCoordinate { x: 0, y: 0 }
                .direction_to_adjacent_coordinate(BoardCoordinate { x: 0, y: 1 }),
            CardinalDirection::South
        );
        assert_eq!(
            BoardCoordinate { x: 0, y: 0 }
                .direction_to_adjacent_coordinate(BoardCoordinate { x: -1, y: 0 }),
            CardinalDirection::West
        );
    }

    #[test]
    #[should_panic]
    fn test_direction_to_adjacent_coordinate_panics_on_non_adjacent() {
        BoardCoordinate { x: 0, y: 0 }
            .direction_to_adjacent_coordinate(BoardCoordinate { x: 1, y: 1 });
    }

    #[test]
    fn test_generates_surrounding_coordinates() {
        let coordinates = BoardCoordinate { x: 2, y: 2 }.surrounding_coordinates();

        assert_eq!(coordinates, vec![
            BoardCoordinate { x: 1, y: 1 },
            BoardCoordinate { x: 1, y: 2 },
            BoardCoordinate { x: 1, y: 3 },
            BoardCoordinate { x: 2, y: 1 },
            BoardCoordinate { x: 2, y: 3 },
            BoardCoordinate { x: 3, y: 1 },
            BoardCoordinate { x: 3, y: 2 },
            BoardCoordinate { x: 3, y: 3 }
        ]);
    }

    #[test]
    fn test_rotate_cardinal_direction() {
        assert_eq!(CardinalDirection::North.rotate(0), CardinalDirection::North);
        assert_eq!(CardinalDirection::North.rotate(1), CardinalDirection::East);
        assert_eq!(
            CardinalDirection::NorthNorthWest.rotate(2),
            CardinalDirection::NorthNorthWest.compass_opposite()
        );
    }

    #[test]
    fn test_adjacent_cardinal_direction() {
        assert_eq!(
            CardinalDirection::North.adjacent(),
            (
                CardinalDirection::NorthNorthWest,
                CardinalDirection::NorthNorthEast
            )
        );
        assert_eq!(
            CardinalDirection::EastSouthEast.adjacent(),
            (CardinalDirection::East, CardinalDirection::SouthSouthEast)
        );
    }
}
