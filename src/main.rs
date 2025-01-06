use crate::tile::{BoardCoordinate, PlacedTile, RenderStyle, TilePlacement};
use crate::tile_definitions::THREE_WAY_JUNCTION_WITH_CITY;

mod tile;
mod tile_definitions;
mod board;
mod player;
mod regions;
mod deck;

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
