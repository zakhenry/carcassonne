use std::collections::{HashMap, HashSet};
use crate::player::{Meeple, Player, RegionIndex};
use crate::regions::{ConnectedRegion, UniqueTileRegion};
use crate::tile::{BoardCoordinate, PlacedTile, RegionType, RenderStyle, TileDefinition, TilePlacement};


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
    // @todo consider IndexMap to preserve insertion order
    placed_tiles: HashMap<BoardCoordinate, PlacedTile>,
    region_index: HashMap<UniqueTileRegion, ConnectedRegion>,
    score_record: Vec<HashMap<Player, u32>>,
    placed_meeple: HashMap<PlacedTile, Meeple>,
    liberated_meeple: HashMap<PlacedTile, Meeple>,
    current_score: HashMap<Player, Vec<RegionScore>>
}

pub(crate) struct MoveHint {
    pub(crate) tile_placement: TilePlacement,
    meeple_placement: Option<RegionIndex>,
    // @todo score
}

impl Board {
    pub(crate) fn get_move_hints(&self, tile: &TileDefinition) -> Vec<MoveHint> {

        let possible_coordinates = self.possible_next_tile_placements();

        if let Some(coordinate) = possible_coordinates.into_iter().next() {

            // @todo implement valid move hinting
            vec![MoveHint {
                tile_placement: TilePlacement {
                    coordinate,
                    rotations: 0,
                },
                meeple_placement: None,
            }]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn new(players: Vec<Player>) -> Self {
        Self {
            players,
            ..Default::default()
        }
    }

    pub(crate) fn new_with_tiles(tiles: Vec<PlacedTile>, players: Vec<Player>) -> Self {
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

    pub(crate) fn place_tile(&mut self, tile: PlacedTile) {
        self.placed_tiles.insert(tile.placement.coordinate, tile);
    }

    fn possible_next_tile_placements(&self) -> HashSet<BoardCoordinate> {

        if self.placed_tiles.is_empty() {
            return HashSet::from([BoardCoordinate::new(0, 0)])
        }

        let mut visited: HashSet<BoardCoordinate> = self.placed_tiles.keys().cloned().collect();

        let mut possible_placements: HashSet<BoardCoordinate> = HashSet::new();

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

    pub(crate) fn render(&self) -> String {
        let mut min_x= i8::MAX;
        let mut min_y = i8::MAX;
        let mut max_x= i8::MIN;
        let mut max_y = i8::MIN;

        for &BoardCoordinate { x, y } in self.placed_tiles.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let mut output = Vec::with_capacity(((max_y - min_y) * 7) as usize);
        output.extend(std::iter::repeat(String::new()).take(output.capacity()));

        for (row_idx, row) in (min_y..max_y).enumerate() {
            for column in min_x..max_x {
                let coord = BoardCoordinate {x: column, y: row};

                let lines = if let Some(tile) = self.placed_tiles.get(&coord) {
                    tile.render_to_lines(RenderStyle::TrueColor)
                } else {
                    vec![std::iter::repeat(' ').take(14).collect();7]
                };

                for (render_row, render) in lines.iter().enumerate() {

                    let row_idx = row_idx * 7 + render_row;

                    output.get_mut(row_idx).unwrap().push_str(render);
                }
            }
        }

        output.join("\n")
    }
}
