use crate::tile::{Expansion, TileDefinition};
use crate::tile_definitions::{ALL_TILE_DEFINITIONS, RIVER_TERMINATOR};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

struct BaseTileSequence {
    tiles: Vec<&'static TileDefinition>,
}

impl BaseTileSequence {
    fn new(rng: &mut impl Rng) -> Self {

        let mut tiles: Vec<_> = ALL_TILE_DEFINITIONS.iter()
            .filter(|t|t.expansion.is_none())
            .flat_map(|t|vec![t; t.count as usize])
            .collect();

        tiles.shuffle(rng);

        Self {
            tiles,
        }
    }
}

impl Iterator for BaseTileSequence {
    type Item = &'static TileDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        let tile = self.tiles.pop();
        tile
    }
}

struct RiverTileSequence {
    tiles: Vec<&'static TileDefinition>,
    current_index: usize,
    river_exhausted: bool,
}

impl RiverTileSequence {
    fn new(rng: &mut impl Rng) -> Self {

        let mut tiles: Vec<_> = ALL_TILE_DEFINITIONS.iter()
            .filter(|t|matches!(t.expansion, Some(Expansion::River)) && t != &&RIVER_TERMINATOR)
            .flat_map(|t|vec![t; t.count as usize])
            .collect();

        tiles.shuffle(rng);

        Self {
            tiles,
            current_index: 0,
            river_exhausted: false
        }
    }
}

impl Iterator for RiverTileSequence {
    type Item = &'static TileDefinition;

    fn next(&mut self) -> Option<Self::Item> {

        if self.current_index == 0 {
            self.current_index += 1;
            Some(&RIVER_TERMINATOR)
        } else if self.river_exhausted {
            None
        } else {
            let tile = self.tiles.pop();
            if tile.is_none() {
                self.river_exhausted = true;
                Some(&RIVER_TERMINATOR)
            } else {
                self.current_index += 1;
                tile
            }
        }

    }
}

struct Deck {
    river_tiles: Option<Box<dyn Iterator<Item = &'static TileDefinition>>>,
    base_tiles: Box<dyn Iterator<Item = &'static TileDefinition>>,
    river_exhausted: bool,
}

impl Deck {
    fn new(include_river: bool, mut rng: impl Rng) -> Self {

        Self {
            river_tiles: if include_river { Some(Box::new(RiverTileSequence::new(&mut rng))) } else { None },
            base_tiles: Box::new(BaseTileSequence::new(&mut rng)),
            river_exhausted: !include_river,
        }
    }
}

impl Iterator for Deck {
    type Item = &'static TileDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.river_exhausted {

            if let Some(ref mut river_sequence) = self.river_tiles {

                if let Some(next_river) = river_sequence.next() {
                    return Some(next_river);
                } else {
                    self.river_exhausted = true;
                }

            }

        }

        self.base_tiles.next()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_deck_yields_only_base_tiles() {

        let rng = StdRng::seed_from_u64(0);

        let base_deck = Deck::new(false, rng);

        for tile in base_deck {
            assert!(tile.expansion.is_none())
        }

    }

    #[test]
    fn deck_yields_different_results_for_different_seeds() {

        let rng_1 = StdRng::seed_from_u64(1);
        let rng_1_copy = StdRng::seed_from_u64(1);
        let rng_2 = StdRng::seed_from_u64(2);

        let base_deck_1: Vec<&'static str> = Deck::new(false, rng_1).map(|t|t.name).collect();
        let base_deck_1_copy: Vec<&'static str> = Deck::new(false, rng_1_copy).map(|t|t.name).collect();
        let base_deck_2: Vec<&'static str> = Deck::new(false, rng_2).map(|t|t.name).collect();

        assert_eq!(base_deck_1, base_deck_1_copy);
        assert_ne!(base_deck_1, base_deck_2);

    }

    #[test]
    fn river_starts_and_ends_with_terminator() {

        let mut rng = StdRng::seed_from_u64(0);
        let river_tile_names: Vec<&'static str> = RiverTileSequence::new(&mut rng).map(|t|t.name).collect();

        assert_eq!(river_tile_names.first().unwrap(), &"River terminator");
        assert_eq!(river_tile_names.last().unwrap(), &"River terminator");
        assert_eq!(river_tile_names.len(), 12)

    }

    #[test]
    fn deck_with_river_starts_with_all_river_tiles() {

        let rng = StdRng::seed_from_u64(0);

        let river_deck = Deck::new(true, rng);

        let (river_tiles, base_tiles): (Vec<_>, Vec<_>) = river_deck.enumerate().map(|(idx, tile)|(idx, &tile.expansion)).partition(|(_, expansion)| matches!(expansion, Some(Expansion::River)));

        let max_river = river_tiles.into_iter().map(|(idx, _)| idx).max().unwrap();
        let min_base = base_tiles.into_iter().map(|(idx, _)| idx).min().unwrap();

        assert!(max_river < min_base)
    }
}
