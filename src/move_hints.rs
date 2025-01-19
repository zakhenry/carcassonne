use crate::board::{Board, TilePlacementSuccess};
use crate::player::{Meeple, MeepleColor, Player, RegionIndex};
use crate::score::Score;
use crate::tile::{BoardCoordinate, PlacedTile, RegionType, TileDefinition, TilePlacement};
use indexmap::IndexSet;
use std::collections::HashSet;
use std::ops::{Add, Sub};
use rayon::prelude::*;

pub(crate) struct MoveHint {
    pub(crate) tile: &'static TileDefinition,
    pub(crate) tile_placement: TilePlacement,
    pub(crate) meeple_placement: Option<RegionIndex>,
}

impl Board {

    fn possible_next_tile_coordinates(&self) -> HashSet<BoardCoordinate> {
        if self.placed_tiles.is_empty() {
            return HashSet::from([BoardCoordinate::new(0, 0)]);
        }

        let board_coordinates: HashSet<BoardCoordinate> = self.placed_tiles.keys().cloned().collect();

        let expanded_coordinates: HashSet<_> = self.placed_tiles.keys()
            .flat_map(|coordinate|coordinate.adjacent_coordinates().into_values())
            .collect();

        expanded_coordinates.sub(&board_coordinates)
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

            let mut region_sequences: HashSet<[RegionType;12]> = HashSet::new();

            (0..4).filter(move |&rotations| {
                let region_sequence = tile.list_oriented_region_types(rotations);

                region_sequences.insert(region_sequence)
            }).map(move |rotations| TilePlacement {
                coordinate,
                rotations,
            })
        });

        candidate_tile_placements.par_bridge().flat_map(|placement| {
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

    pub(crate) fn score_delta(&self, board: &Board, player: &Player, calculate_as_if_last_tile: bool) -> Score {
        let mut test_board = board.clone();

        let dummy_tile = PlacedTile {
            tile: self.tile,
            placement: self.tile_placement.clone(),
            meeple: self.meeple_placement.map(|region_index|(region_index, Meeple { color: player.meeple_color })),
        };

        let TilePlacementSuccess {score_delta, ..} = test_board.place_tile(dummy_tile).expect("should be a valid move");

        if calculate_as_if_last_tile {

            let before = board.calculate_board_score();
            let after = test_board.calculate_board_score();

            (after + score_delta) - before
        } else {
            score_delta
        }

    }

}

#[cfg(test)]
mod tests {
    use crate::test_util::tests::{TestMoveHint, TestPlayer};
    use crate::tile::RenderStyle;
    use super::*;
    use crate::tile_definitions::{CENTRE_CITY_WITH_PENNANT, CLOISTER_IN_FIELD, CLOISTER_WITH_ROAD, CORNER_ROAD, CORNER_ROAD_WITH_SIDE_CITY, SIDE_CITY, STRAIGHT_CITY_WITH_SIDE_FIELDS, STRAIGHT_ROAD};


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

    #[test]
    fn should_return_no_valid_meeple_placement_when_all_possible_places_are_taken() {

        let mut alice = Player::blue();

        Board::new_with_tiles([
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 3, 1),
        ])
            .expect("should be valid")
            .get_move_hints(&CENTRE_CITY_WITH_PENNANT, true)
            .should_have_hint_placements([
                "1,0 @0"
            ]);

    }

    #[test]
    fn should_return_the_relative_score_change_for_each_possible_move() {

        let mut alice = Player::red();
        let mut bob = Player::blue();
        let mut carol = Player::green();

        let board = Board::new_with_tiles([
            alice.move_with_meeple(&SIDE_CITY, 1, 0, 1, 1),
            bob.move_no_meeple(&CORNER_ROAD, 1, -1, 3),
            carol.move_no_meeple(&CORNER_ROAD, 1, 1, 0),
            alice.move_with_meeple(&SIDE_CITY, 0, -1, 0, 1),
            bob.move_no_meeple(&SIDE_CITY, 0, 1, 2),
            carol.move_with_meeple(&CORNER_ROAD, -1, -1, 2, 1),
            alice.move_with_meeple(&CLOISTER_IN_FIELD, -1, 1, 0, 1),
            bob.move_with_meeple(&SIDE_CITY, -1, 0, 3, 1),
        ])
            .expect("should be valid");

        println!("{}", board.render(&RenderStyle::Ascii));

        let move_hints = board
            .get_move_hints(&CENTRE_CITY_WITH_PENNANT, true);

        move_hints.should_have_hint_placements([
            "0,0 @0"
        ]);

        let hint = move_hints.first().expect("should be one");

        assert!(hint.meeple_placement.is_none());

        assert_eq!(hint.score_delta(&board, &carol, false), Score::from_iter([
            (&alice, 12), // closed city (12)
        ]));

        assert_eq!(hint.score_delta(&board, &carol, true), Score::from_iter([
            (&alice, 11), // closed city (12) - 2 already factored city tiles + cloister addition (1)
            (&bob, -1), // city shutout by alice
            (&carol, 3), // farmer gained closed city
        ]));

    }

}
