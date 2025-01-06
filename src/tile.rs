use std::collections::HashMap;
use std::fmt::Debug;
use colored::{Color, ColoredString, Colorize};

pub const TILE_WIDTH: usize = 7;

#[derive(Debug)]
pub(crate) struct TileCoordinate {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BoardCoordinate {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

#[derive(Debug)]
pub struct TilePlacement {
    pub(crate) coordinate: BoardCoordinate,
    pub(crate) rotations: u8, // count of 90° rotations from the definition (i.e. range is 0-3 inclusive)
}

// note that the diagonal corners are intentionally omitted because carcassonne tiles do not form
// connected regions from touching corners
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TileDefinition {
    pub(crate) count: u8,
    pub(crate) name: &'static str,
    pub(crate) render: TileRenderRepresentation,
    pub(crate) regions: &'static [Region],
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum RenderCell {
    Field,
    Road,
    City,
    Cloister,
    Pennant,
    Water,
    Corner, // @todo remove
}

impl RenderCell {

    fn colors(&self) -> (Color, Color, Color, Color) {
        match self {
            Self::Field => (Color::Green, Color::TrueColor { r: 143, g: 185, b: 45}, Color::TrueColor { r: 165, g: 184, b: 90}, Color::TrueColor { r: 85, g: 122, b: 30}),
            Self::Road => (Color::BrightBlack, Color::TrueColor { r: 190, g: 190, b: 190}, Color::TrueColor { r: 220, g: 220, b: 220}, Color::TrueColor { r: 150, g: 150, b: 150}),
            Self::City => (Color::Yellow, Color::TrueColor { r: 199, g: 147, b: 88}, Color::TrueColor { r: 208, g: 169, b: 116}, Color::TrueColor { r: 154, g: 94, b: 56}),
            Self::Cloister => (Color::BrightWhite, Color::Red, Color::Red, Color::Red),
            Self::Pennant => (Color::Red, Color::Red, Color::Red, Color::Red),
            Self::Water => (Color::Blue, Color::Red, Color::Red, Color::Red),
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


#[derive(Debug)]
pub struct Board {
    placed_tiles: HashMap<BoardCoordinate, PlacedTile>,
}

impl Board {
    pub(crate) fn new(tiles: Vec<PlacedTile>) -> Self {
        Self {
            placed_tiles: HashMap::from_iter(
                tiles
                    .into_iter()
                    .map(|t| (t.placement.coordinate.clone(), t)),
            ),
        }
    }

    pub fn placed_tile_count(&self) -> usize {
        self.placed_tiles.len()
    }
}


#[derive(Debug)]
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

