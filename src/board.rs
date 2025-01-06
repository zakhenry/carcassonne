use std::collections::HashMap;
use crate::player::{Meeple, Player};
use crate::regions::{ConnectedRegion, UniqueTileRegion};
use crate::tile::{BoardCoordinate, PlacedTile, RegionType};


// private val regionIndex: MutableMap<UniqueTileRegion, ConnectedRegion> = mutableMapOf(),
// private val scoreRecord: MutableList<Map<Player, Int>> = mutableListOf(),
// private val placedMeeple: MutableMap<PlacedTile, Meeple> = mutableMapOf(),
// private val liberatedMeeple: MutableMap<PlacedTile, Meeple> = mutableMapOf(),
// private var currentScore: Map<Player, List<RegionScore>> = players.associateWith { emptyList() },

#[derive(Debug)]
struct RegionScore {
    region: RegionType,
    score: u32
}

#[derive(Debug)]
#[derive(Default)]
pub struct Board {
    players: Vec<Player>,
    placed_tiles: HashMap<BoardCoordinate, PlacedTile>,
    region_index: HashMap<UniqueTileRegion, ConnectedRegion>,
    score_record: Vec<HashMap<Player, u32>>,
    placed_meeple: HashMap<PlacedTile, Meeple>,
    liberated_meeple: HashMap<PlacedTile, Meeple>,
    current_score: HashMap<Player, Vec<RegionScore>>
}

impl Board {
    pub(crate) fn new(tiles: Vec<PlacedTile>, players: Vec<Player>) -> Self {
        Self {
            players,
            placed_tiles: HashMap::from_iter(
                tiles
                    .into_iter()
                    .map(|t| (t.placement.coordinate.clone(), t)),
            ),
            ..Default::default()
        }
    }

    pub fn placed_tile_count(&self) -> usize {
        self.placed_tiles.len()
    }
}
