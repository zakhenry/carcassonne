use crate::board::Board;
use crate::deck::Deck;
use crate::player::Player;
use crate::tile::{BoardCoordinate, PlacedTile, RenderStyle, TilePlacement};
use crate::tile_definitions::THREE_SIDED_CITY_WITH_ROAD;
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

mod board;
mod deck;
mod game_logic;
mod player;
mod tile;
mod tile_definitions;
mod connected_regions;

fn main() {
    let rng = Rc::new(RefCell::new(StdRng::from_entropy()));

    let players = vec![Player::black(), Player::green()];

    let mut board = Arc::new(RwLock::new(Board::new(players)));

    let board_clone = Arc::clone(&board);

    let mut deck = Deck::new(true, rng, move |tile| {
        !board_clone.read().unwrap().get_move_hints(tile).is_empty()
    });

    for tile in deck {
        let move_hints = board.read().unwrap().get_move_hints(tile);

        // create dense board by selecting hints that maximize adjacent placement of tiles
        let selected_move_hint = move_hints.iter().max_by_key(|&hint| {
            board
                .read()
                .unwrap()
                .list_adjacent_tiles(&hint.tile_placement.coordinate)
                .iter()
                .filter_map(|(_, t)| *t)
                .count()
        });

        if let Some(random_move) = selected_move_hint {
            let tile = PlacedTile {
                tile,
                placement: random_move.tile_placement.clone(),
            };

            // println!("{}", tile.render_to_lines(RenderStyle::TrueColor).join("\n"));

            board.write().unwrap().place_tile(tile).unwrap();
        } else {
            eprintln!("no move hints?")
        }
    }

    println!(
        "Done, board has {} tiles placed",
        board.read().unwrap().placed_tile_count()
    );

    println!("{}", board.read().unwrap().render(RenderStyle::TrueColor));

    println!("{:?}", board.read().unwrap().get_connected_regions())
}
