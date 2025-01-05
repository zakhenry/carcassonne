use crate::tile::{Board, BoardCoordinate, PlacedTile, TilePlacement};
use tile_definitions::CROSS_INTERSECTION;

mod tile;
mod tile_definitions;

fn main() {
    let board = Board::new(vec![PlacedTile {
        tile: &CROSS_INTERSECTION,
        placement: TilePlacement {
            coordinate: BoardCoordinate { x: 0, y: 0 },
            rotations: 0,
        },
    }]);

    println!("{}", board.placed_tile_count())
}
