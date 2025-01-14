use std::collections::HashSet;
use std::ops::Sub;
use indexmap::IndexSet;
use crate::board::{Board};
use crate::player::{Meeple, MeepleColor, Player, RegionIndex};
use crate::score::Score;
use crate::tile::{BoardCoordinate, PlacedTile, RegionType, TileDefinition, TilePlacement};

pub(crate) struct MoveHint {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) tile_placement: TilePlacement,
    pub(crate) meeple_placement: Option<RegionIndex>,
}

impl Board {

    fn possible_next_tile_coordinates(&self) -> IndexSet<BoardCoordinate> {
        if self.placed_tiles.is_empty() {
            return IndexSet::from([BoardCoordinate::new(0, 0)]);
        }

        let mut visited: HashSet<BoardCoordinate> = self.placed_tiles.keys().cloned().collect();

        let mut possible_placements: IndexSet<BoardCoordinate> = IndexSet::new();

        for coordinate in self.placed_tiles.keys() {
            for adjacent_coordinate in coordinate.adjacent_coordinates().values() {
                if visited.contains(adjacent_coordinate) {
                    continue;
                }

                possible_placements.insert(*adjacent_coordinate);
                visited.insert(*adjacent_coordinate);
            }

            visited.insert(*coordinate);
        }

        possible_placements
    }

    pub(crate) fn get_move_hints(
        &self,
        tile: &'static TileDefinition,
        include_meeple_placement_hints: bool,
    ) -> Vec<MoveHint> {
        let possible_coordinates = self.possible_next_tile_coordinates();

        let candidate_tile_placements = possible_coordinates.into_iter().flat_map(|coordinate| {
            // here we discard duplicate rotated sequences as these represent tiles with rotational
            // symmetry so it doesn't make sense to offer it as a placement variant

            let mut region_sequences: HashSet<Vec<RegionType>> = HashSet::new();

            (0..4).filter(move |&rotations| {
                let region_sequence = tile.list_oriented_region_types(rotations);

                region_sequences.insert(region_sequence)
            }).map(move |rotations| TilePlacement {
                coordinate,
                rotations,
            })
        });

        candidate_tile_placements.flat_map(|placement| {
            let unplaced_meeple_candidate = [(placement.clone(), None)];

            if include_meeple_placement_hints {
                (0..tile.regions.len()).map(|idx| (placement.clone(), Some(RegionIndex::new(idx)))).chain(unplaced_meeple_candidate).collect::<Vec<_>>()
            } else {
                unplaced_meeple_candidate.into_iter().collect::<Vec<_>>()
            }
        }).filter(|(placement, meeple_region_index)| {
            self.validate_tile_placement(
                &PlacedTile {
                    tile,
                    placement: placement.clone(),
                    meeple: meeple_region_index.map(|idx| (idx, Meeple::dummy())),
                },
                None,
            ).is_ok()
        }).map(|(tile_placement, meeple_placement)|MoveHint {
            tile,
            tile_placement,
            meeple_placement,
        }).collect()
    }

}


impl MoveHint {

    pub(crate) fn score_delta(&self, board: &Board, player: &Player) -> Score {
        let before = board.calculate_board_score();

        let mut test_board = board.clone();

        let dummy_tile = PlacedTile {
            tile: self.tile,
            placement: self.tile_placement.clone(),
            meeple: self.meeple_placement.map(|region_index|(region_index, Meeple { color: MeepleColor::Black, player_id: player.id.clone() })),
        };

        test_board.place_tile(dummy_tile).expect("should be a valid move");

        let after = test_board.calculate_board_score();

        after - before
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_definitions::{STRAIGHT_RIVER, THREE_SIDED_CITY_WITH_ROAD};

    #[test]
    fn test_base_deck_yields_only_base_tiles() {

    }

}
