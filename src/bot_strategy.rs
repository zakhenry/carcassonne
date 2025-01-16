use crate::board::Board;
use crate::move_hints::MoveHint;
use crate::player::Player;
use crate::tile::{PlacedTile, TileDefinition};

pub trait Bot {
    fn select_hint(&self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint>;
}

/// This bot is only interested in filling gaps in the grid. It otherwise places meeples and tiles
/// at random
pub(crate) struct FillTheGridBot;

impl Bot for FillTheGridBot {
    fn select_hint(&self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {

        let move_hints = board.get_move_hints(tile, true);

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
pub(crate) struct MyopicBot;

impl Bot for MyopicBot {
    fn select_hint(&self, board: &Board, player: &Player, tile: &'static TileDefinition) -> Option<MoveHint> {
        let move_hints = board.get_move_hints(tile, true);

        move_hints.into_iter().max_by_key(|hint|{

            let score = hint.score_delta(board, player, true);

            score.get_player(player).copied()

        })

    }
}
