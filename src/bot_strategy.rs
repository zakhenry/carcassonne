use std::ops::Deref;
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::rngs::StdRng;
use crate::board::Board;
use crate::move_hints::MoveHint;
use crate::player::Player;
use crate::tile::{PlacedTile, TileDefinition};

pub trait Bot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint>;
}

#[derive(Clone)]
pub struct BotPlayer {
    pub(crate) player: Player,
    pub(crate) bot: BotStrategy
}

impl Player {
    pub(crate) fn with_bot(self, bot: BotStrategy) -> BotPlayer {
        BotPlayer { player: self, bot }
    }
}

#[derive(Clone)]
pub enum BotStrategy {
    Rando(RandoBot),
    Myopic(MyopicBot),
    FillTheGrid(FillTheGridBot),
    Jerk(JerkBot),
    ScoreRanking(ScoreRankingBot),
    Lazy(LazyBot),
}

impl Bot for BotStrategy {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        match self {
            BotStrategy::Rando(b)  => b.select_hint(board, player, tile),
            BotStrategy::Myopic(b)  => b.select_hint(board, player, tile),
            BotStrategy::FillTheGrid(b) => b.select_hint(board, player, tile),
            BotStrategy::Jerk(b) => b.select_hint(board, player, tile),
            BotStrategy::ScoreRanking(b) => b.select_hint(board, player, tile),
            BotStrategy::Lazy(b) => b.select_hint(board, player, tile),
        }
    }
}

/// This bot picks a hint entirely at random
#[derive(Clone)]
pub(crate) struct RandoBot(StdRng);

impl RandoBot {
    pub(crate) fn new(rng: StdRng) -> Self {
        Self(rng)
    }
}

impl Bot for RandoBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        let mut move_hints = board.get_move_hints(tile, true);
        move_hints.shuffle(&mut self.0);
        move_hints.pop()
    }

}

/// This bot is only interested in filling gaps in the grid. It otherwise places meeples and tiles
/// at random
#[derive(Clone)]
pub(crate) struct FillTheGridBot(StdRng);

impl FillTheGridBot {
    pub(crate) fn new(rng: StdRng) -> Self {
        Self(rng)
    }
}

impl Bot for FillTheGridBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {

        let mut move_hints = board.get_move_hints(tile, true);

        move_hints.shuffle(&mut self.0);

        move_hints.into_iter().max_by_key(|hint| {

            let adjacent_region_count = board
                .list_adjacent_tiles(&hint.tile_placement.coordinate)
                .iter()
                .filter_map(|(_, t)| *t)
                .count();

            let meeple_placement = match hint.meeple_placement {
                Some(_) => 1,
                None => 0,
            };

            adjacent_region_count + meeple_placement
        })

    }

}

/// This bot looks only at its own score change on a single tile placement; ignoring all other
/// player scores
#[derive(Clone)]
pub(crate) struct MyopicBot;

impl Bot for MyopicBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        let move_hints = board.get_move_hints(tile, true);

        move_hints.into_iter().max_by_key(|hint|{

            let score = hint.score_delta(board, player, true);

            score.get_player(player).copied()

        })

    }
}


#[derive(Clone)]
pub(crate) struct ScoreRankingBot;

impl Bot for ScoreRankingBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        todo!()
    }
}

/// This bot looks only at how it can make other's score worse. It won't place meeple otherwise
#[derive(Clone)]
pub(crate) struct JerkBot;

impl Bot for JerkBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        let move_hints = board.get_move_hints(tile, true);

        move_hints.into_iter().max_by_key(|hint|{

            let score = hint.score_delta(board, player, true);

            let mut weight = 0;

            for (player_id, score) in score.iter() {
                if player_id != &player.meeple_color {
                    weight += score
                }
            }

            let meeple_modifier = if hint.meeple_placement.is_some() && weight > 0 {
                1
            } else {
                0
            };

            (weight, meeple_modifier)
        })

    }
}


/// This bot finds the first valid move it spots, biasing to place meeple (otherwise it would never score)
#[derive(Clone)]
pub(crate) struct LazyBot;

impl Bot for LazyBot {
    fn select_hint(&mut self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        board.get_move_hints(tile, true).into_iter().max_by_key(|hint|{
            if hint.meeple_placement.is_some() {
                1
            } else {
                0
            }
        })
    }
}
