use crate::tile::{Expansion, TileDefinition};
use crate::tile_definitions::{ALL_TILE_DEFINITIONS, RIVER_TERMINATOR};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

struct BaseTileSequence {
    tiles: Vec<&'static TileDefinition>,
    tile_can_be_placed: Box<dyn Fn(&'static TileDefinition) -> bool>,
    rng: Rc<RefCell<StdRng>>,
}

impl BaseTileSequence {
    fn new<F>(rng: Rc<RefCell<StdRng>>, tile_can_be_placed: F) -> Self
    where
        F: Fn(&'static TileDefinition) -> bool + 'static,
    {
        let mut tiles: Vec<_> = ALL_TILE_DEFINITIONS
            .iter()
            .filter(|t| t.expansion.is_none())
            .flat_map(|t| vec![t; t.count as usize])
            .collect();

        tiles.shuffle(rng.borrow_mut().deref_mut());

        Self {
            tiles,
            tile_can_be_placed: Box::new(tile_can_be_placed),
            rng,
        }
    }
}

impl Iterator for BaseTileSequence {
    type Item = &'static TileDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tile = self.tiles.pop();

        let mut discarded_tiles: Vec<&TileDefinition> = Vec::new();

        loop {
            if (self.tile_can_be_placed)(tile?) {
                break;
            } else {
                discarded_tiles.push(tile?);
                tile = self.tiles.pop();
            }
        }

        // once we find a tile, if we had discarded any tiles before we shuffle them back
        // into the stack
        if !discarded_tiles.is_empty() {
            self.tiles.append(&mut discarded_tiles);
            self.tiles.shuffle(self.rng.borrow_mut().deref_mut());
        }

        tile
    }
}

struct RiverTileSequence {
    tiles: Vec<&'static TileDefinition>,
    current_index: usize,
    river_exhausted: bool,
    tile_can_be_placed: Box<dyn Fn(&'static TileDefinition) -> bool>,
    rng: Rc<RefCell<StdRng>>,
}

impl RiverTileSequence {
    fn new<F>(rng: Rc<RefCell<StdRng>>, tile_can_be_placed: F) -> Self
    where
        F: Fn(&'static TileDefinition) -> bool + 'static,
    {
        let mut tiles: Vec<_> = ALL_TILE_DEFINITIONS
            .iter()
            .filter(|t| matches!(t.expansion, Some(Expansion::River)) && t != &&RIVER_TERMINATOR)
            .flat_map(|t| vec![t; t.count as usize])
            .collect();

        tiles.shuffle(rng.borrow_mut().deref_mut());

        Self {
            tiles,
            current_index: 0,
            river_exhausted: false,
            tile_can_be_placed: Box::new(tile_can_be_placed),
            rng,
        }
    }
}

impl Iterator for RiverTileSequence {
    type Item = &'static TileDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == 0 {
            self.current_index += 1;
            return Some(&RIVER_TERMINATOR);
        }

        if self.river_exhausted {
            return None;
        }

        let mut tile = self.tiles.pop();

        if tile.is_none() {
            self.river_exhausted = true;
            return Some(&RIVER_TERMINATOR);
        }

        // The official ruling from Hans im Glück is "Try to think while playing. Players may have
        // an unfinished River...but it‘s their own fault."
        if !(self.tile_can_be_placed)(tile?) {
            return None;
        }

        self.current_index += 1;
        tile
    }
}

pub struct Deck {
    river_tiles: Option<Box<dyn Iterator<Item = &'static TileDefinition>>>,
    base_tiles: BaseTileSequence,
    river_exhausted: bool,
}

impl Deck {
    pub(crate) fn new<F>(
        include_river: bool,
        rng: Rc<RefCell<StdRng>>,
        tile_can_be_placed: F,
    ) -> Self
    where
        F: Fn(&'static TileDefinition) -> bool + Clone + 'static,
    {
        Self {
            river_tiles: if include_river {
                Some(Box::new(RiverTileSequence::new(
                    rng.clone(),
                    tile_can_be_placed.clone(),
                )))
            } else {
                None
            },
            base_tiles: BaseTileSequence::new(rng, tile_can_be_placed),
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
    use crate::tile_definitions::{STRAIGHT_RIVER, THREE_SIDED_CITY_WITH_ROAD};

    #[test]
    fn test_base_deck_yields_only_base_tiles() {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));

        let base_deck = Deck::new(false, rng, |_| true);

        for tile in base_deck {
            assert!(tile.expansion.is_none())
        }
    }

    #[test]
    fn deck_yields_different_results_for_different_seeds() {
        let rng_1 = Rc::new(RefCell::new(StdRng::seed_from_u64(1)));
        let rng_1_copy = Rc::new(RefCell::new(StdRng::seed_from_u64(1)));
        let rng_2 = Rc::new(RefCell::new(StdRng::seed_from_u64(2)));

        let base_deck_1: Vec<&'static str> =
            Deck::new(false, rng_1, |_| true).map(|t| t.name).collect();
        let base_deck_1_copy: Vec<&'static str> = Deck::new(false, rng_1_copy, |_| true)
            .map(|t| t.name)
            .collect();
        let base_deck_2: Vec<&'static str> =
            Deck::new(false, rng_2, |_| true).map(|t| t.name).collect();

        assert_eq!(base_deck_1, base_deck_1_copy);
        assert_ne!(base_deck_1, base_deck_2);
    }

    #[test]
    fn river_starts_and_ends_with_terminator() {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));
        let river_tile_names: Vec<&'static str> = RiverTileSequence::new(rng, |_| true)
            .map(|t| t.name)
            .collect();

        assert_eq!(river_tile_names.first().unwrap(), &"River terminator");
        assert_eq!(river_tile_names.last().unwrap(), &"River terminator");
        assert_eq!(river_tile_names.len(), 12)
    }

    #[test]
    fn deck_with_river_starts_with_all_river_tiles() {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));

        let river_deck = Deck::new(true, rng, |_| true);

        let (river_tiles, base_tiles): (Vec<_>, Vec<_>) = river_deck
            .enumerate()
            .map(|(idx, tile)| (idx, &tile.expansion))
            .partition(|(_, expansion)| matches!(expansion, Some(Expansion::River)));

        let max_river = river_tiles.into_iter().map(|(idx, _)| idx).max().unwrap();
        let min_base = base_tiles.into_iter().map(|(idx, _)| idx).min().unwrap();

        assert!(max_river < min_base)
    }

    #[test]
    fn river_skips_tiles_that_cannot_be_placed() {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));
        let river_tile_names: Vec<&'static str> =
            RiverTileSequence::new(rng, |tile| tile.name != STRAIGHT_RIVER.name)
                .map(|t| t.name)
                .collect();

        assert!(!river_tile_names.contains(&STRAIGHT_RIVER.name));
    }

    #[test]
    fn deck_completes_with_missing_tiles_if_one_cannot_be_placed() {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));

        let test_tile = THREE_SIDED_CITY_WITH_ROAD;

        assert_eq!(test_tile.count, 1);

        let mut deck = Deck::new(true, rng, |tile| {
            tile.name != THREE_SIDED_CITY_WITH_ROAD.name
        });

        let board_tiles: Vec<_> = deck.by_ref().collect();

        assert_eq!(board_tiles.len(), 83);
        assert!(!board_tiles.contains(&&test_tile))
    }
}
