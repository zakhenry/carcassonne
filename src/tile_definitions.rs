use crate::tile::CardinalDirection::{
    East, EastNorthEast, EastSouthEast, North, NorthNorthEast, NorthNorthWest, South,
    SouthSouthEast, SouthSouthWest, West, WestNorthWest, WestSouthWest,
};
use crate::tile::{Region, RenderCell, TileCoordinate, TileDefinition, TileRenderRepresentation};

pub const CROSS_INTERSECTION: TileDefinition = TileDefinition {
    count: 1,
    name: "Cross intersection",
    render: ascii_to_tile(
        "
FFFRFFF
FFFRFFF
FFFRFFF
RRRFRRR
FFFRFFF
FFFRFFF
FFFRFFF
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::Road {
            edges: &[South],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
        },
        Region::Road {
            edges: &[East],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
        Region::Road {
            edges: &[West],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Field {
            edges: &[EastSouthEast, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 4, y: 4 },
        },
        Region::Field {
            edges: &[SouthSouthWest, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 2, y: 4 },
        },
    ],
};

pub const ALL_TILE_DEFINITIONS: [TileDefinition;1] = [CROSS_INTERSECTION];

const fn ascii_to_tile(ascii: &'static str) -> TileRenderRepresentation {
    let mut repr: [[RenderCell; 7]; 7] = [[RenderCell::Road; 7]; 7];

    // @todo once iterators are allowed in const fns, the following can be greatly simplified to
    // the following approach
    // for (row, line) in ascii.trim().lines().enumerate() {
    //     for (column, cell) in line.chars().enumerate() {
    //         let value = match cell {
    //             'F' => RenderCell::Field,
    //             'R' => RenderCell::Road,
    //             'C' => RenderCell::City,
    //             'T' => RenderCell::Cloister,
    //             v => panic!("Unexpected cell representation [{}] at [{}, {}]", v, column, row)
    //         };
    //
    //         repr[row][column] = value;
    //     }
    // }

    let mut i = 0;
    let mut row = 0;
    let mut column = 0;

    while i < ascii.len() {
        let byte = ascii.as_bytes()[i];

        if byte < 128 {
            i += 1;
        } else {
            panic!("Invalid UTF-8 sequence")
        }

        if byte == b'\n' {
            if column == 0 {
                continue;
            }
            column = 0;
            row += 1;
            continue;
        }

        if column == 7 {
            panic!("Count of characters in row exceeds expected 7");
        }

        let value = match byte {
            b'F' => RenderCell::Field,
            b'R' => RenderCell::Road,
            b'C' => RenderCell::City,
            b'T' => RenderCell::Cloister,
            _ => panic!("Unexpected cell representation"),
        };

        repr[row][column] = value;

        column += 1;
    }

    TileRenderRepresentation(repr)
}
