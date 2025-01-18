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
use std::time::Instant;
use indexmap::IndexMap;
use rand::rngs::OsRng;
use uuid::Uuid;
use crate::bot_strategy::{Bot, BotPlayer, BotStrategy, FillTheGridBot, JerkBot, LazyBot, MyopicBot, RandoBot};
use crate::score::Score;
use base64::{engine::general_purpose, Engine as _};


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

// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {

    let seed: [u8; 32] = OsRng.gen();
    // let seed = [210, 233, 120, 7, 69, 3, 119, 55, 175, 78, 62, 244, 9, 228, 209, 19, 30, 87, 10, 94, 40, 240, 237, 33, 213, 63, 135, 34, 17, 176, 193, 162];

    let seed_string = general_purpose::URL_SAFE.encode(&seed);

    println!("{}", seed_string);

    let rng = Rc::new(RefCell::new(StdRng::from_seed(seed)));

    let jerk_bot = BotStrategy::Jerk(JerkBot);
    let fill_the_grid_bot = BotStrategy::FillTheGrid(FillTheGridBot::new(StdRng::from_rng(rng.borrow_mut().deref_mut()).unwrap()));
    let rando_bot = BotStrategy::Rando(RandoBot::new(StdRng::from_rng(rng.borrow_mut().deref_mut()).unwrap()));
    let myopic_bot = BotStrategy::Myopic(MyopicBot);
    let lazy_bot = BotStrategy::Lazy(LazyBot);

    let alice = Player::red().with_name("Alice").with_bot(lazy_bot.clone());
    let bob = Player::green().with_name("Bob").with_bot(lazy_bot);
    let carol = Player::blue().with_name("Carol").with_bot(rando_bot);
    let dave = Player::yellow().with_name("Dave").with_bot(jerk_bot);

    let mut overall_score = Score::new();

    let iteration_count = 100;
    let render_style = RenderStyle::TrueColor;

    // let alice_rando = Player::red().with_name("Alice").with_bot(rando_bot.clone());
    // let bob_rando = Player::green().with_name("Bob").with_bot(rando_bot.clone());
    let players: IndexMap<_, BotPlayer> = vec![
        // alice_rando,
        // bob_rando,
        alice,
        bob,
        // carol,
        // dave
    ]
        .into_iter()
        .map(|p| (p.player.meeple_color, p))
        .collect();

    let now = Instant::now();

    for _ in 0..iteration_count {

        let mut players = players.clone();


        let player_ids: Vec<_> = players.keys().copied().collect();
        let mut player_id_iter = player_ids.iter().cycle();

        let mut score = Score::new();
        let board = Arc::new(RwLock::new(Board::new()));

        let board_clone = Arc::clone(&board);

        let deck = Deck::new(true, rng.clone(), move |tile| {
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

            let BotPlayer { player,bot } = players.get_mut(player_id).expect("should exist");

            let selected_move_hint = bot.select_hint(&board.read().unwrap(), &player, tile);

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
                    players.get_mut(&meeple.color).expect("should exist").player.meeple.push(meeple);
                }
            } else {
                // panic!("no move hints?")
            }
        }
        //
        // println!(
        //     "Done, board has {} tiles placed",
        //     board.read().unwrap().placed_tile_count()
        // );

        let board_score = board.read().unwrap().calculate_board_score();

        score += board_score;

        // println!("{}", board.read().unwrap().render(&render_style));
        // println!("Final score is:\n{}", score.render(&players.into_iter().map(|(id, BotPlayer { player, ..})|(id, player)).collect(), &render_style));

        overall_score += score;
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("Completed {} iterations. Final score is\n{}", iteration_count, overall_score.render(&players.into_iter().map(|(id, BotPlayer { player, ..})|(id, player)).collect(), &render_style))

}
