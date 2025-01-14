use std::collections::{HashMap, HashSet};
use std::ops::{AddAssign, Sub};
use colored::Colorize;
use crate::board::Board;
use crate::connected_regions::ConnectedRegion;
use crate::player::{MeepleColor, Player, PlayerId};
use crate::tile::{Region, RegionType, RenderStyle};

#[derive(Debug, Default, PartialEq)]
pub struct Score(HashMap<PlayerId, i32>);

impl Score {
    pub(crate) fn new() -> Self {
        Self(Default::default())
    }


    pub(crate) fn from_iter<'a, I: IntoIterator<Item=(&'a Player, i32)>>(player_score: I) -> Self {
        Self(HashMap::from_iter(player_score.into_iter().map(|(player, score)|(player.id, score))))
    }

    // @todo make a proper pretty table
    pub(crate) fn render(&self, players: &HashMap<PlayerId, Player>, render_style: &RenderStyle) -> String {

        let mut out = String::new();

        for (player_id, score) in &self.0 {

            let player = players.get(player_id).expect("should exist");

            out += format!("{} = {} \n", &player.name.clone().unwrap_or_else(|| "unnamed".parse().unwrap()).color(player.meeple_color.render_color(render_style)), score).as_str();
        }

        out

    }

    pub(crate) fn add_score(&mut self, player_id: PlayerId, score: i32) {
        *self.0.entry(player_id).or_insert(0) += score;
    }
}


impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        for (player_id, score) in rhs.0.iter() {
            self.add_score(*player_id, *score);
        }
    }
}


impl Sub for Score {
    type Output = Score;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for (player_id, score) in rhs.0.iter() {
            *self.0.entry(*player_id).or_insert(0) -= score;
        }

        self
    }
}

impl ConnectedRegion {

    pub(crate) fn score(&self, board: &Board) -> u32 {
        match self.region_type {
            RegionType::City => {

                let base_score = self.tile_regions.iter().map(|region| match region.region {
                    Region:: City { pennant: true, .. } => 2,
                    Region:: City { pennant: false, .. } => 1,
                    _ => unreachable!()
                }).sum();

                if self.is_closed() {
                    base_score * 2
                } else {
                    base_score
                }

            },
            RegionType::Field => {

                let adjacent_closed_city_count = self.adjacent_regions.iter().filter(|&connected_region_id|{
                    if let Some(region) = board.get_connected_region(connected_region_id) {
                        region.region_type == RegionType::City && region.is_closed()
                    } else {
                        false
                    }
                }).count();

                (adjacent_closed_city_count * 3) as u32

            },
            RegionType::Cloister => {

                assert_eq!(self.tile_regions.len(), 1);
                let cloister_coordinate = self.tile_regions.iter().map(|r|r.tile_position).next().expect("there should be one");

                let adjacent_count = board.list_surrounding_tiles(&cloister_coordinate).len();

                adjacent_count as u32 + 1
            },
            RegionType::Road => self.tile_regions.len() as u32,
            RegionType::Water => 0
        }
    }

    pub(crate) fn majority_meeple_player_ids(&self, board: &Board) -> Vec<PlayerId> {
        let mut counts = HashMap::new();

        for player_id in self.residents(board).iter().map(|(_, _, &ref meeple)| meeple.player_id) {
            *counts.entry(player_id).or_insert(0) += 1;
        }

        if let Some(&max_count) = counts.values().max() {
            // Collect all PlayerIds with the maximum count
            counts
                .into_iter()
                .filter_map(|(player_id, count)| if count == max_count { Some(player_id) } else { None })
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Board {

    /// Note this finds the score of the current board state; it ignores any previous score delta
    /// caused by meeple being liberated
    pub fn calculate_board_score(&self) -> Score {
        let mut score_delta = Score::new();

        for connected_region in self.get_connected_regions() {
            let region_score = connected_region.score(self);
            for winning_player in connected_region.majority_meeple_player_ids(self) {
                score_delta.add_score(winning_player, region_score as i32);
            }
        }

        score_delta

    }
}

#[cfg(test)]
mod tests {
    use crate::board::TilePlacementSuccess;
    use crate::player::{Meeple, RegionIndex};
    use crate::tile::{PlacedTile, TileDefinition};
    use crate::tile_definitions::{CLOISTER_IN_FIELD, CLOISTER_WITH_ROAD, CORNER_CITY_WITH_PENNANT, CORNER_ROAD, CORNER_ROAD_WITH_CORNER_CITY, OPPOSING_SIDE_CITIES, SIDE_CITY, STRAIGHT_ROAD, THREE_SIDED_CITY};
    use super::*;

    trait TestUtil {
        fn should_have_score(&self, expectation: Score);
    }

    impl TestUtil for [PlacedTile] {
        fn should_have_score(&self, expectation: Score) {
            let mut board = Board::default();

            let mut score = Score::new();

            for tile in self.iter().cloned() {
                let TilePlacementSuccess { score_delta, .. } = board.place_tile(tile).expect("tile placement should be valid");
                score += score_delta
            }

            score += board.calculate_board_score();

            println!("{}", board.render(&RenderStyle::Ascii));

            assert_eq!(score, expectation)
        }
    }

    trait TestPlayer {

        fn move_with_meeple(&mut self, tile: &'static TileDefinition, x: i8, y: i8, rotations: u8, meeple_region_index: usize) -> PlacedTile;

        fn move_no_meeple(&self, tile: &'static TileDefinition, x: i8, y: i8, rotations: u8) -> PlacedTile;
    }

    impl TestPlayer for Player {
        fn move_with_meeple(&mut self, tile: &'static TileDefinition, x: i8, y: i8, rotations: u8, meeple_region_index: usize) -> PlacedTile {
            let mut tile = PlacedTile::new(tile, x, y, rotations);

            tile.meeple = Some((RegionIndex::new(meeple_region_index), self.meeple.pop().expect("player should have enough meeple")));

            tile
        }

        fn move_no_meeple(&self, tile: &'static TileDefinition, x: i8, y: i8, rotations: u8) -> PlacedTile {
            PlacedTile::new(tile, x, y, rotations)
        }
    }


    #[test]
    fn should_add_score() {

        let alice = Player::green();
        let bob = Player::blue();

        let mut a = Score::from_iter([(&alice, 1)]);
        let b = Score::from_iter([(&alice, 2), (&bob, 3)]);

        a += b;

        assert_eq!(a, Score::from_iter([(&alice, 3), (&bob, 3)]))
    }

    #[test]
    fn should_subtract_scores() {
        let alice = Player::green();
        let bob = Player::blue();
        let carol = Player::blue();

        let a = Score::from_iter([(&alice, 3), (&carol, 4)]);
        let b = Score::from_iter([(&alice, 2), (&bob, 3)]);

        assert_eq!(a - b, Score::from_iter([(&alice, 1), (&bob, -3), (&carol, 4)]))
    }

    #[test]
    fn should_score_roads_based_on_number_of_connected_roads() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            bob.move_no_meeple(&CORNER_ROAD, -1, -1, 0),
            alice.move_with_meeple(&STRAIGHT_ROAD, -1, 0, 0, 0),
            bob.move_no_meeple(&CORNER_ROAD, -1, 1, 3),
            alice.move_no_meeple(&STRAIGHT_ROAD, 0, -1, 1),
            bob.move_with_meeple(&CORNER_ROAD, 0, 0, 0, 2),
        ].should_have_score(Score::from_iter([
            (&alice, 4),
            (&bob, 1),
        ]))

    }


    #[test]
    fn should_score_cloisters_based_on_number_of_surrounding_tiles() {


        let mut alice = Player::red();
        let mut bob = Player::green();
        let mut carol = Player::blue();

        [
            alice.move_with_meeple(&CLOISTER_IN_FIELD, -1, -1, 0, 1),
            bob.move_with_meeple(&CLOISTER_WITH_ROAD, -1, 0, 2, 2),
            carol.move_no_meeple(&CORNER_ROAD, -1, 1, 3),
            alice.move_no_meeple(&STRAIGHT_ROAD, 0, 1, 3),
        ].should_have_score(Score::from_iter([
            (&alice, 2),
            (&bob, 4),
            // (&carol, 0),
        ]))

    }

    #[test]
    fn should_score_cities_based_on_number_of_connected_cities_with_pennants_scoring_and_additional_point() {
        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 1),
            bob.move_with_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2, 1),
            alice.move_no_meeple(&THREE_SIDED_CITY, 0, 1, 3),
            bob.move_no_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 2),
            (&bob, 3),
        ]))
    }


    #[test]
    fn should_score_cities_with_double_points_when_they_are_closed() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 1),
            bob.move_with_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2, 1),
            alice.move_no_meeple(&THREE_SIDED_CITY, 0, 1, 3),
            bob.move_no_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 1),
            alice.move_no_meeple(&SIDE_CITY, 0, 2, 2),
            bob.move_no_meeple(&SIDE_CITY, 1, 1, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 8),
            (&bob, 3),
        ]))

    }


    #[test]
    fn should_score_fields_based_on_number_of_completed_adjacent_cities() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 0),
            bob.move_no_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2),
            alice.move_no_meeple(&THREE_SIDED_CITY, 0, 1, 3),
            bob.move_with_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 1, 1),
            alice.move_no_meeple(&SIDE_CITY, 0, 2, 2),
            bob.move_no_meeple(&SIDE_CITY, 1, 1, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 3),
            (&bob, 0),
        ]))

    }

    #[test]
    fn should_allocate_equal_points_between_players_meeple_equally_sharing_a_region() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 0),
            bob.move_no_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2),
            alice.move_no_meeple(&THREE_SIDED_CITY, 0, 1, 3),
            bob.move_with_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 1, 1),
            alice.move_no_meeple(&SIDE_CITY, 0, 2, 2),
            bob.move_no_meeple(&SIDE_CITY, 1, 1, 1),
            alice.move_no_meeple(&OPPOSING_SIDE_CITIES, 2, 1, 0),
            bob.move_no_meeple(&STRAIGHT_ROAD, 3, 1, 0),
            bob.move_no_meeple(&CORNER_ROAD, 3, 0, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 3),
            (&bob, 3),
        ]))

    }

    #[test]
    fn should_only_give_the_score_for_one_meeple_when_a_player_has_more_than_one_meeple_in_a_region() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 0),
            bob.move_no_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2),
            alice.move_with_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 1, 1),
            bob.move_no_meeple(&THREE_SIDED_CITY, 0, 1, 3),
            alice.move_no_meeple(&SIDE_CITY, 0, 2, 2),
            bob.move_no_meeple(&SIDE_CITY, 1, 1, 1),
            alice.move_no_meeple(&OPPOSING_SIDE_CITIES, 2, 1, 0),
            bob.move_no_meeple(&STRAIGHT_ROAD, 3, 1, 0),
            bob.move_no_meeple(&CORNER_ROAD, 3, 0, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 3),
        ]))

    }


    #[test]
    fn should_give_all_points_to_the_player_with_a_region_dominated_by_their_meeple() {

        let mut alice = Player::red();
        let mut bob = Player::green();

        [
            alice.move_with_meeple(&SIDE_CITY, 0, 0, 0, 0),
            bob.move_no_meeple(&CORNER_CITY_WITH_PENNANT, 1, 0, 2),
            alice.move_with_meeple(&THREE_SIDED_CITY, 0, 1, 3, 0),
            bob.move_with_meeple(&CORNER_ROAD_WITH_CORNER_CITY, 2, 0, 2, 1),
            alice.move_no_meeple(&SIDE_CITY, 0, 2, 2),
            bob.move_no_meeple(&CORNER_ROAD, 2, 1, 3),
            alice.move_no_meeple(&STRAIGHT_ROAD, -1, 1, 0),
            bob.move_no_meeple(&SIDE_CITY, 1, 1, 1),
            alice.move_no_meeple(&CORNER_ROAD, -1, 0, 1),
        ].should_have_score(Score::from_iter([
            (&alice, 3),
        ]))

    }


}
