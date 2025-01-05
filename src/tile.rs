use std::collections::HashMap;
use std::fmt::Debug;
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
}

#[derive(Debug)]
pub struct PlacedTile {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) placement: TilePlacement, // @todo meeple
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
}

// Definitions copied from https://cad.onshape.com/documents/04cfee738b84b4699685349a/w/f6c7a218fb2ae3244c5e18ee/e/e45463d6dd17036cc38b1be6

#[derive(Debug)]
pub(crate) struct TileRenderRepresentation(pub [[RenderCell; 7]; 7]);
