use crate::board::{Board, TilePlacementSuccess};
use crate::deck::Deck;
use crate::player::Player;
use crate::tile::{PlacedTile, RenderStyle};
use rand::prelude::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use indexmap::IndexMap;
use rand::rngs::OsRng;
use crate::bot_strategy::{Bot, FillTheGridBot, MyopicBot, RandoBot};
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
mod bot_strategy;

fn main() {

    let seed: [u8; 32] = OsRng.gen();
    // let seed = [210, 233, 120, 7, 69, 3, 119, 55, 175, 78, 62, 244, 9, 228, 209, 19, 30, 87, 10, 94, 40, 240, 237, 33, 213, 63, 135, 34, 17, 176, 193, 162];

    println!("{:?}", &seed);

    let rng = Rc::new(RefCell::new(StdRng::from_seed(seed)));

    let mut alice = Player::red();
    alice.name = Some("Alice".to_string());
    let mut bob = Player::green();
    bob.name = Some("Bob".to_string());
    let mut carol = Player::blue();
    carol.name = Some("Carol".to_string());

    let render_style = RenderStyle::TrueColor;

    let alice_bot = (alice, Box::new(MyopicBot) as Box<dyn Bot>);
    let bob_bot = (bob, Box::new(FillTheGridBot::new(StdRng::from_rng(rng.borrow_mut().deref_mut()).unwrap())) as Box<dyn Bot>);
    let carol_bot = (carol, Box::new(RandoBot::new(StdRng::from_rng(rng.borrow_mut().deref_mut()).unwrap())) as Box<dyn Bot>);

    let mut players: IndexMap<_, (Player, Box<dyn Bot>)> = vec![alice_bot, bob_bot, carol_bot]
        .into_iter()
        .map(|p| (p.0.meeple_color, p))
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

        let (player,bot_strategy) = players.get_mut(player_id).expect("should exist");

        let selected_move_hint = bot_strategy.select_hint(&board.read().unwrap(), &player, tile);

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
                players.get_mut(&meeple.color).expect("should exist").0.meeple.push(meeple);
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
    println!("Final score is:\n{}", score.render(&players.into_iter().map(|(id, (player, _))|(id, player)).collect(), &render_style));
}
