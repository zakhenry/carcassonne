use crate::board::{Board, TilePlacementSuccess};
use crate::deck::Deck;
use crate::player::Player;
use crate::tile::{PlacedTile, RenderStyle};
use rand::prelude::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use rand::rngs::OsRng;
use crate::score::Score;

mod board;
mod connected_regions;
mod deck;
mod game_logic;
mod player;
mod tile;
mod tile_definitions;
mod score;
mod move_hints;
mod test_util;

fn main() {

    let seed: [u8; 32] = OsRng.gen();
    // let seed = [210, 233, 120, 7, 69, 3, 119, 55, 175, 78, 62, 244, 9, 228, 209, 19, 30, 87, 10, 94, 40, 240, 237, 33, 213, 63, 135, 34, 17, 176, 193, 162];

    println!("{:?}", &seed);

    let rng = Rc::new(RefCell::new(StdRng::from_seed(seed)));

    let mut alice = Player::blue();
    alice.name = Some("Alice".to_string());
    let mut bob = Player::red();
    bob.name = Some("Bob".to_string());

    let render_style = RenderStyle::TrueColor;

    let mut players: HashMap<_, _> = vec![alice, bob]
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    let player_ids: Vec<_> = players.keys().copied().collect();
    let mut player_id_iter = player_ids.iter().cycle();

    let mut score = Score::new();
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

            let score_delta = hint.score_delta(&board.read().unwrap(), &player);

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

            let TilePlacementSuccess { liberated_meeple, score_delta } = board.write().unwrap().place_tile(tile).unwrap();

            score += score_delta;

            for meeple in liberated_meeple {
                players.get_mut(&meeple.player_id).expect("should exist").meeple.push(meeple);
            }
        } else {
            panic!("no move hints?")
        }
    }

    println!(
        "Done, board has {} tiles placed",
        board.read().unwrap().placed_tile_count()
    );

    let board_score = board.read().unwrap().calculate_board_score();

    score += board_score;

    println!("{}", board.read().unwrap().render(&render_style));
    println!("Final score is:\n{}", score.render(&players, &render_style));
}
