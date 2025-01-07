use crate::deck::Deck;
use crate::tile::{BoardCoordinate, PlacedTile, RenderStyle, TilePlacement};
use crate::tile_definitions::THREE_SIDED_CITY_WITH_ROAD;
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use crate::board::Board;
use crate::player::Player;

mod tile;
mod tile_definitions;
mod board;
mod player;
mod regions;
mod deck;
mod game_logic;

fn main() {
    let rng = Rc::new(RefCell::new(StdRng::from_entropy()));

    let players = vec![Player::black(), Player::green()];

    let mut board = Arc::new(RwLock::new(Board::new(players)));

    let board_clone = Arc::clone(&board);

    let mut deck = Deck::new(true, rng, move |tile| !board_clone.read().unwrap().get_move_hints(tile).is_empty());

    for tile in deck {

        let move_hint = board.read().unwrap().get_move_hints(tile).into_iter().next();

        if let Some(random_move) = move_hint {

            let tile = PlacedTile {
                tile,
                placement: random_move.tile_placement,
            };

            println!("{}", tile.render_to_lines(RenderStyle::TrueColor).join("\n"));

            board.write().unwrap().place_tile(tile)

        } else {
            eprintln!("no move hints?")
        }

    }

    println!("Done, board has {} tiles placed", board.read().unwrap().placed_tile_count());

    println!("{}", board.read().unwrap().render())

}
