use std::cell::RefCell;
use crate::player::{Meeple, Player, RegionIndex};
use crate::tile::{
    BoardCoordinate, CardinalDirection, PlacedTile, RegionType, RenderStyle, TileDefinition,
    TilePlacement, TILE_WIDTH,
};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use indexmap::IndexMap;
use crate::connected_regions::{ConnectedRegion, ConnectedRegionId, PlacedTileEdge, PlacedTileRegion};
use crate::tile_definitions::RIVER_TERMINATOR;
// private val regionIndex: MutableMap<UniqueTileRegion, ConnectedRegion> = mutableMapOf(),
// private val scoreRecord: MutableList<Map<Player, Int>> = mutableListOf(),
// private val placedMeeple: MutableMap<PlacedTile, Meeple> = mutableMapOf(),
// private val liberatedMeeple: MutableMap<PlacedTile, Meeple> = mutableMapOf(),
// private var currentScore: Map<Player, List<RegionScore>> = players.associateWith { emptyList() },

#[derive(Debug)]
struct RegionScore {
    region: RegionType,
    score: u32,
}

#[derive(Debug, Default)]
pub struct Board {
    // players: Vec<Player>,
    placed_tiles: IndexMap<BoardCoordinate, PlacedTile>,
    connected_regions: HashMap<ConnectedRegionId, ConnectedRegion>,
    region_index: HashMap<PlacedTileEdge, ConnectedRegionId>,
    score_record: Vec<HashMap<Player, u32>>,
    placed_meeple: HashMap<PlacedTile, Meeple>,
    liberated_meeple: HashMap<PlacedTile, Meeple>,
    current_score: HashMap<Player, Vec<RegionScore>>,
}

pub(crate) struct MoveHint {
    pub(crate) tile_placement: TilePlacement,
    pub(crate) meeple_placement: Option<RegionIndex>,
    // @todo score
}


#[derive(Debug)]
pub enum InvalidTilePlacement {
    TileAlreadyAtCoordinate,
    TileDoesNotContactPlacedTiles,
    TileEdgesDoNotMatchPlacedTiles,
    OtherMeepleAlreadyInConnectedRegion,
    RiverMustBeConnected,
    RiverMustNotImmediatelyTurnOnItself,
}

#[derive(Debug, Default)]
pub struct TilePlacementSuccess {
    score_delta: HashMap<Player, u32>,
    placed_meeple: Option<Meeple>,
    liberated_meeple: Vec<Meeple>
}

impl Board {
    pub(crate) fn get_move_hints(&self, tile: &'static TileDefinition) -> Vec<MoveHint> {
        let possible_coordinates = self.possible_next_tile_coordinates();

        let candidate_tile_placements = possible_coordinates.into_iter().flat_map(|coordinate| {
            // here we discard duplicate rotated sequences as these represent tiles with rotational
            // symmetry so it doesn't make sense to offer it as a placement variant

            let mut region_sequences: HashSet<Vec<RegionType>> = HashSet::new();

            (0..4)
                .into_iter()
                .filter(move |&rotations| {
                    let region_sequence = tile.list_oriented_region_types(rotations);

                    region_sequences.insert(region_sequence)
                })
                .map(move |rotations| TilePlacement {
                    coordinate,
                    rotations,
                })
        });

        candidate_tile_placements
            .flat_map(|placement|{
                (0..tile.regions.len()).map(|idx|(placement.clone(), Some(RegionIndex::new(idx)))).chain([(placement.clone(), None)]).collect::<Vec<_>>()
            })
            .filter(|(placement, meeple_region_index)| self.validate_tile_placement(tile, &placement, meeple_region_index).is_ok())
            .map(|(tile_placement, meeple_placement)| MoveHint {
                tile_placement,
                meeple_placement,
            })
            .collect()
    }

    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn new_with_tiles(/*players: Vec<Player>, */tiles: Vec<PlacedTile>) -> Result<Self, InvalidTilePlacement> {

        let mut board = Board::default();

        for tile in tiles {
            board.place_tile(tile)?;
        }

        Ok(board)
    }

    pub fn placed_tile_count(&self) -> usize {
        self.placed_tiles.len()
    }

    pub(crate) fn place_tile(&mut self, tile: PlacedTile) -> Result<TilePlacementSuccess, InvalidTilePlacement> {
        let meeple_region_index = tile.meeple.clone().map(|m|m.0);
        self.validate_tile_placement(tile.tile, &tile.placement, &meeple_region_index)?;

        let tile_connected_regions = ConnectedRegion::from_tile(&tile);

        for mut connected_region in tile_connected_regions {
            let mut regions_to_merge: HashSet<ConnectedRegionId> = Default::default();

            for edge in connected_region.connected_edges.keys() {
                let opposite = edge.opposing_tile_edge();
                if let Some(connected_region_id) = self.region_index.get(&opposite) {
                    regions_to_merge.insert(connected_region_id.clone());
                }
            }

            for region_id in regions_to_merge {
                let merge_region = self.connected_regions.remove(&region_id).expect("should exist");

                connected_region.merge_mut(merge_region).expect("should merge");
            }

            for placed_tile_edge in connected_region.connected_edges.keys() {
                self.region_index.insert(placed_tile_edge.clone(), connected_region.id);
            }

            self.connected_regions.insert(connected_region.id, connected_region);

        }


        self.placed_tiles.insert(tile.placement.coordinate, tile);

        // @todo implement scoring and meeple tracking in success result
        Ok(TilePlacementSuccess::default())
    }

    fn opposing_tile_edge(&self, edge: &PlacedTileEdge) -> Option<PlacedTileEdge> {
        todo!()
    }

    fn possible_next_tile_coordinates(&self) -> HashSet<BoardCoordinate> {
        if self.placed_tiles.is_empty() {
            return HashSet::from([BoardCoordinate::new(0, 0)]);
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

    pub(crate) fn validate_tile_placement(
        &self,
        tile: &'static TileDefinition,
        placement: &TilePlacement,
        meeple_region_index: &Option<RegionIndex>,
    ) -> Result<(), InvalidTilePlacement> {
        // empty board is always valid for placement of a tile
        if self.placed_tiles.is_empty() {
            return Ok(());
        }

        if self.placed_tiles.contains_key(&placement.coordinate) {
            return Err(InvalidTilePlacement::TileAlreadyAtCoordinate);
        }

        let surrounding_regions = self.get_surrounding_regions(&placement.coordinate);

        if surrounding_regions.iter().all(|region| region.is_none()) {
            return Err(InvalidTilePlacement::TileDoesNotContactPlacedTiles);
        }

        let candidate_placed_tile = PlacedTile { tile, placement: placement.clone(), meeple: meeple_region_index.map(|idx|(idx, Meeple::dummy())) };

        let own_regions = tile.list_oriented_region_types(placement.rotations);

        let region_pairings: Vec<_> = own_regions
            .iter()
            .zip(surrounding_regions)
            .collect();

        if region_pairings.iter().any(|(own_region, neighbor_region)| match neighbor_region {
                Some(region) if &region != own_region => true,
                _ => false,
            })
        {
            return Err(InvalidTilePlacement::TileEdgesDoNotMatchPlacedTiles);
        }

        if own_regions.iter().any(|r|matches!(r, RegionType::Water)) {
            let paired_water = region_pairings.iter().filter(|(own_region, neighbor_region)| match (neighbor_region, own_region) {
                (Some(RegionType::Water), RegionType::Water) => true,
                _ => false,
            }).count();

            if paired_water < 1 {
                return Err(InvalidTilePlacement::RiverMustBeConnected);
            }

            if tile != &RIVER_TERMINATOR {

                let (_, prev_tile) = self.placed_tiles.last().expect("There should always be a last tile");

                let direction_to_prev = placement.coordinate.direction_to_adjacent_coordinate(prev_tile.placement.coordinate);


                let current_heading = candidate_placed_tile.get_opposite_river_end_direction(direction_to_prev);
                let previous_source = prev_tile.get_opposite_river_end_direction(direction_to_prev.compass_opposite());

                if previous_source == current_heading {
                    return Err(InvalidTilePlacement::RiverMustNotImmediatelyTurnOnItself);
                }
            }


        }

        // @todo meeple validation

        Ok(())
    }

    pub(crate) fn list_adjacent_tiles(
        &self,
        board_coordinate: &BoardCoordinate,
    ) -> Vec<(CardinalDirection, Option<&PlacedTile>)> {
        board_coordinate
            .adjacent_coordinates()
            .into_iter()
            .map(|(direction, coordinate)| (direction, self.placed_tiles.get(&coordinate)))
            .collect()
    }

    fn get_surrounding_regions(
        &self,
        board_coordinate: &BoardCoordinate,
    ) -> Vec<Option<RegionType>> {
        self.list_adjacent_tiles(board_coordinate)
            .iter()
            .flat_map(|(direction, tile)| {
                if let Some(adjacent_tile) = tile {
                    adjacent_tile
                        .list_regions_on_edge(&direction.compass_opposite())
                        .iter()
                        .map(|region_type| Some(region_type.clone()))
                        .collect()
                } else {
                    vec![None; 3]
                }
            })
            .collect()
    }

    pub(crate) fn get_connected_regions(&self) -> Vec<&ConnectedRegion> {
        self.connected_regions.values().collect()
    }

    pub(crate) fn render(&self, style: RenderStyle) -> String {
        if self.placed_tiles.is_empty() {
            return "[Empty board]".to_string();
        }

        let mut min_x = i8::MAX;
        let mut min_y = i8::MAX;
        let mut max_x = i8::MIN;
        let mut max_y = i8::MIN;

        for &BoardCoordinate { x, y } in self.placed_tiles.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let mut output = Vec::with_capacity(((max_y - min_y) + 1) as usize * TILE_WIDTH);
        // note we can't pre-allocate the width of the board as the color control chars make each
        // row a different length depending on what regions are represented
        output.extend(std::iter::repeat(String::new()).take(output.capacity()));

        for (row_idx, row) in (min_y..=max_y).enumerate() {
            for column in min_x..=max_x {
                let coord = BoardCoordinate { x: column, y: row };

                let lines = if let Some(tile) = self.placed_tiles.get(&coord) {
                    tile.render_to_lines(&style)
                } else {
                    vec![std::iter::repeat(' ').take(TILE_WIDTH * 2).collect(); TILE_WIDTH]
                };

                for (render_row, render) in lines.iter().enumerate() {
                    let row_idx = row_idx * TILE_WIDTH + render_row;

                    output.get_mut(row_idx).unwrap().push_str(render);
                }
            }
        }

        output.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::RegionType::{Field, Road, Water};
    use crate::tile_definitions::{CORNER_RIVER, CORNER_ROAD, STRAIGHT_RIVER, STRAIGHT_ROAD};

    #[test]
    fn test_valid_on_first_tile() {
        let board = Board::new();

        assert!(board
            .validate_tile_placement(
                &STRAIGHT_ROAD,
                &TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
            )
            .is_ok())
    }

    #[test]
    fn test_invalid_if_tile_already_at_coordinate() {
        let board = Board::new_with_tiles(
            vec![PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: HashMap::new()
            }],
        ).unwrap();

        let res = board.validate_tile_placement(
            &STRAIGHT_ROAD,
            &TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::TileAlreadyAtCoordinate)
        ))
    }

    #[test]
    fn test_invalid_if_tile_does_not_contact_placed_tiles() {
        let board = Board::new_with_tiles(
            vec![PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: HashMap::new()
            }],
        ).unwrap();

        let res = board.validate_tile_placement(
            &STRAIGHT_ROAD,
            &TilePlacement {
                coordinate: BoardCoordinate { x: 2, y: 0 },
                rotations: 0,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::TileDoesNotContactPlacedTiles)
        ))
    }

    #[test]
    fn test_get_surrounding_regions() {
        let board = Board::new_with_tiles(
            vec![
                PlacedTile {
                    tile: &STRAIGHT_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: 0 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
                PlacedTile {
                    tile: &CORNER_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: -1 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
                PlacedTile {
                    tile: &CORNER_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 1, y: -1 },
                        rotations: 1,
                    },
                    meeple: HashMap::new()
                },
            ],
        ).unwrap();

        // println!("{}", board.render());

        let res = board.get_surrounding_regions(&BoardCoordinate { x: 1, y: 0 });

        assert_eq!(
            res,
            vec![
                Some(Field),
                Some(Road),
                Some(Field),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Field),
                Some(Field),
                Some(Field)
            ]
        )
    }

    #[test]
    fn test_invalid_if_tile_edges_do_not_match_placed_tiles() {
        let board = Board::new_with_tiles(
            vec![
                PlacedTile {
                    tile: &STRAIGHT_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: 0 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
                PlacedTile {
                    tile: &CORNER_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: -1 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
                PlacedTile {
                    tile: &CORNER_ROAD,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 1, y: -1 },
                        rotations: 1,
                    },
                    meeple: HashMap::new()
                },
            ],
        ).unwrap();

        let res = board.validate_tile_placement(
            &STRAIGHT_ROAD,
            &TilePlacement {
                coordinate: BoardCoordinate { x: 1, y: 0 },
                rotations: 1,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::TileEdgesDoNotMatchPlacedTiles)
        ))
    }


    #[test]
    fn test_invalid_if_river_is_disconnected() {
        let board = Board::new_with_tiles(
            vec![
                PlacedTile {
                    tile: &STRAIGHT_RIVER,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: 0 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
            ],
        ).unwrap();

        let res = board.validate_tile_placement(
            &STRAIGHT_RIVER,
            &TilePlacement {
                coordinate: BoardCoordinate { x: 1, y: 0 },
                rotations: 0,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::RiverMustBeConnected)
        ))
    }
    #[test]
    fn test_invalid_if_river_turns_on_itself() {
        let board = Board::new_with_tiles(
            vec![
                PlacedTile {
                    tile: &CORNER_RIVER,
                    placement: TilePlacement {
                        coordinate: BoardCoordinate { x: 0, y: 0 },
                        rotations: 0,
                    },
                    meeple: HashMap::new()
                },
            ],
        ).unwrap();

        let res = board.validate_tile_placement(
            &CORNER_RIVER,
            &TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: -1 },
                rotations: 3,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::RiverMustNotImmediatelyTurnOnItself)
        ))
    }
}
