use crate::board::Board;
use crate::deck::Deck;
use crate::player::Player;
use crate::tile::{PlacedTile, RenderStyle};
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

mod board;
mod connected_regions;
mod deck;
mod game_logic;
mod player;
mod tile;
mod tile_definitions;

fn main() {
    let rng = Rc::new(RefCell::new(StdRng::from_entropy()));

    let mut players: HashMap<_, _> = vec![Player::blue(), Player::red()]
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    let player_ids: Vec<_> = players.keys().map(|id| id.clone()).collect();
    let mut player_id_iter = player_ids.iter().cycle();

    let board = Arc::new(RwLock::new(Board::new()));

    let board_clone = Arc::clone(&board);

    let deck = Deck::new(true, rng, move |tile| {
        !board_clone
            .read()
            .unwrap()
            .get_move_hints(tile, false)
            .is_empty()
    });

    for tile in deck {
        let player_id = player_id_iter
            .next()
            .expect("should always have a next player while tiles remain");

        let player = players.get_mut(player_id).expect("should exist");

        let move_hints = board.read().unwrap().get_move_hints(tile, true);

        // create dense board by selecting hints that maximize adjacent placement of tiles
        let selected_move_hint = move_hints.iter().max_by_key(|&hint| {
            let adjacent_region_count = board
                .read()
                .unwrap()
                .list_adjacent_tiles(&hint.tile_placement.coordinate)
                .iter()
                .filter_map(|(_, t)| *t)
                .count();

            let meeple_placement = match hint.meeple_placement {
                Some(_) => 1,
                None => 0,
            };

            adjacent_region_count + meeple_placement
        });

        if let Some(random_move) = selected_move_hint {
            let tile = PlacedTile {
                tile,
                placement: random_move.tile_placement.clone(),
                meeple: if let (Some(region_index), Some(meeple)) =
                    (random_move.meeple_placement, player.meeple.pop())
                {
                    Some((region_index, meeple))
                } else {
                    None
                },
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
}
