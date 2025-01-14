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
    use crate::tile_definitions::{CENTRE_CITY_WITH_PENNANT, CLOISTER_WITH_ROAD, CORNER_ROAD, CORNER_ROAD_WITH_SIDE_CITY, SIDE_CITY, STRAIGHT_RIVER, STRAIGHT_ROAD, THREE_SIDED_CITY_WITH_ROAD};

    trait MoveHintTestUtil {
        fn should_have_hint_placements<T : IntoIterator<Item = &'static str>>(&self, placements: T);
    }

    impl MoveHintTestUtil for Vec<MoveHint> {
        fn should_have_hint_placements<T : IntoIterator<Item = &'static str>>(&self, placements: T) {
            let mut expectation: Vec<_> = placements.into_iter().collect();
            expectation.sort();

            let mut test: Vec<_> = self.iter().map(|hint|
                format!("{},{} @{}", hint.tile_placement.coordinate.x, hint.tile_placement.coordinate.y, hint.tile_placement.rotations)
            ).collect();
            test.sort();

            assert_eq!(test, expectation)
        }
    }

    #[test]
    fn should_return_the_board_origin_when_the_board_is_empty() {

        Board::new()
            .get_move_hints(&CORNER_ROAD_WITH_SIDE_CITY, false)
            .should_have_hint_placements([
                "0,0 @0",
                "0,0 @1",
                "0,0 @2",
                "0,0 @3"
            ])

    }

    #[test]
    fn should_return_no_rotationally_symmetric_move_hints() {

        Board::new()
            .get_move_hints(&CENTRE_CITY_WITH_PENNANT, false)
            .should_have_hint_placements([
                "0,0 @0",
            ]);


        Board::new()
            .get_move_hints(&STRAIGHT_ROAD, false)
            .should_have_hint_placements([
                "0,0 @0",
                "0,0 @1",
            ]);

    }

    #[test]
    fn should_return_a_list_of_valid_tile_placements_for_a_given_tile() {

        Board::new_with_tiles([
            PlacedTile::new(&CLOISTER_WITH_ROAD, 1, 0, 2),
            PlacedTile::new(&CORNER_ROAD, 2, 0, 0),
        ])
            .expect("should be valid")
            .get_move_hints(&CORNER_ROAD_WITH_SIDE_CITY, false)
            .should_have_hint_placements([
                "1,-1 @1",
                "1,1 @1",
                "1,1 @2",
                "0,0 @0",
                "2,-1 @1",
                "3,0 @0",
                "3,0 @1",
                "2,1 @1",
                "2,1 @2",
            ]);

    }

}
