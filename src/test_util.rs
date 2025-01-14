#[cfg(test)]
pub(crate) mod tests {
    use crate::board::{Board, TilePlacementSuccess};
    use crate::move_hints::MoveHint;
    use crate::player::{Player, RegionIndex};
    use crate::score::Score;
    use crate::tile::{PlacedTile, RenderStyle, TileDefinition};

    pub(crate) trait TestMoveHint {
        fn should_have_hint_placements<T : IntoIterator<Item = &'static str>>(&self, placements: T);
    }

    impl TestMoveHint for Vec<MoveHint> {
        fn should_have_hint_placements<T : IntoIterator<Item = &'static str>>(&self, placements: T) {
            let mut expectation: Vec<_> = placements.into_iter().collect();
            expectation.sort();

            let mut test: Vec<_> = self.iter().map(|hint|
                format!("{},{} @{}{}", hint.tile_placement.coordinate.x, hint.tile_placement.coordinate.y, hint.tile_placement.rotations, hint.meeple_placement.map_or("".to_string(), |r|format!(" [{}]", *r)))
            ).collect();
            test.sort();

            assert_eq!(test, expectation)
        }
    }



    pub(crate) trait TestConnectedRegion {
        fn should_have_score(&self, expectation: Score);
    }

    impl TestConnectedRegion for [PlacedTile] {
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

    pub(crate) trait TestPlayer {

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

}
