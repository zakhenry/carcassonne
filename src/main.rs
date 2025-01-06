use crate::tile::{Board, BoardCoordinate, PlacedTile, RenderStyle, TilePlacement};
use tile_definitions::CROSS_INTERSECTION;
use crate::tile_definitions::THREE_WAY_JUNCTION_WITH_CITY;

mod tile;
mod tile_definitions;

fn main() {

    let tile = PlacedTile {
        tile: &THREE_WAY_JUNCTION_WITH_CITY,
        placement: TilePlacement {
            coordinate: BoardCoordinate { x: 0, y: 0 },
            rotations: 1,
        },
    };

    println!("{}", tile.render_to_lines(RenderStyle::TrueColor).join("\n"));

}
