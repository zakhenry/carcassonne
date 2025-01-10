use crate::connected_regions::{
    ConnectedRegion, ConnectedRegionId, PlacedTileEdge,
};
use crate::player::{Meeple, Player, PlayerId, RegionIndex};
use crate::tile::{BoardCoordinate, CardinalDirection, PlacedTile, Region, RegionType, RenderStyle, TileDefinition, TilePlacement, TILE_WIDTH};
use crate::tile_definitions::RIVER_TERMINATOR;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use crate::score::Score;

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
    InvalidMeeplePlacementIndex,
    MeepleCannotBePlacedInRiver
}

#[derive(Debug, Default)]
pub struct TilePlacementSuccess {
    pub score_delta: Score,
    pub liberated_meeple: Vec<Meeple>,
}

impl Board {
    pub(crate) fn get_tile_at_coordinate(
        &self,
        coordinate: &BoardCoordinate,
    ) -> Option<&PlacedTile> {
        self.placed_tiles.get(coordinate)
    }

    pub(crate) fn get_connected_region(
        &self,
        id: &ConnectedRegionId,
    ) -> Option<&ConnectedRegion> {
        self.connected_regions.get(id)
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
        }).map(|(tile_placement, meeple_placement)| MoveHint {
            tile_placement,
            meeple_placement,
        }).collect()
    }

    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn new_with_tiles(
        tiles: Vec<PlacedTile>,
    ) -> Result<Self, InvalidTilePlacement> {
        let mut board = Board::default();

        for tile in tiles {
            board.place_tile(tile)?;
        }

        Ok(board)
    }

    pub fn placed_tile_count(&self) -> usize {
        self.placed_tiles.len()
    }

    fn get_candidate_regions_to_merge(
        &self,
        connected_region: &ConnectedRegion,
    ) -> HashSet<ConnectedRegionId> {
        let mut regions_to_merge: HashSet<ConnectedRegionId> = Default::default();

        for edge in connected_region.connected_edges.keys() {
            let opposite = edge.opposing_tile_edge();
            if let Some(connected_region_id) = self.region_index.get(&opposite) {
                regions_to_merge.insert(*connected_region_id);
            }
        }

        regions_to_merge
    }

    pub(crate) fn place_tile(
        &mut self,
        tile: PlacedTile,
    ) -> Result<TilePlacementSuccess, InvalidTilePlacement> {
        let tile_connected_regions = tile.own_connected_regions();

        self.validate_tile_placement(&tile, Some(&tile_connected_regions))?;

        let mut liberated_meeple: Vec<Meeple> = Vec::new();
        let mut score_delta = Score::new();

        for mut connected_region in tile_connected_regions {
            let regions_to_merge = self.get_candidate_regions_to_merge(&connected_region);

            for region_id in regions_to_merge {
                let merge_region = self.connected_regions.remove(&region_id).expect("should exist");

                connected_region.merge_mut(merge_region).expect("should merge");
            }

            for placed_tile_edge in connected_region.connected_edges.keys() {
                self.region_index.insert(placed_tile_edge.clone(), connected_region.id);
            }

            if connected_region.is_closed() {
                let resident_tile_coordinates: Vec<_> = connected_region.residents(self).iter().map(|(tile, _, _)| tile.placement.coordinate).collect();

                let mut liberated_meeple_for_region = Vec::new();

                for coordinate in resident_tile_coordinates {
                    let tile = self.placed_tiles.get_mut(&coordinate).expect("should exist");

                    if let Some((region_index, _)) = &tile.meeple {
                        if connected_region.tile_regions.iter().any(|r|r.tile_position == tile.placement.coordinate && &r.region_index == region_index) {

                            if let Some((_, meeple)) = tile.meeple.take() {
                                assert_ne!(&connected_region.region_type, &RegionType::Water, "meeple shouldn't need to be liberated from the river. something has gone horribly wrong!");
                                liberated_meeple_for_region.push(meeple);
                            }
                        }
                    }

                }

                if let Some(winning_player) = connected_region.majority_meeple_player_id(self) {
                    score_delta.add_score(winning_player, connected_region.score(self))
                }

                liberated_meeple.extend(liberated_meeple_for_region);
            }

            self.connected_regions.insert(connected_region.id, connected_region);
        }

        let adjacent_closed_priest_tile_coordinates: Vec<_> = self.list_surrounding_tiles(&tile.placement.coordinate).into_iter()
            .filter(|tile|tile.has_occupied_cloister())
            .filter_map(|tile| {
                let adjacent_count = self.list_surrounding_tiles(&tile.placement.coordinate).len();

                // note we consider closed because we haven't yet placed the tile adjacent to this.
                if adjacent_count == 7 {
                    Some(tile.placement.coordinate.clone())
                } else {
                    None
                }
            }).collect();

        self.placed_tiles.insert(tile.placement.coordinate, tile);

        for coordinate in adjacent_closed_priest_tile_coordinates {
            let tile = self.placed_tiles.get_mut(&coordinate).expect("should exist");
            if let Some((_, meeple)) = tile.meeple.take() {
                score_delta.add_score(meeple.player_id, 8);
                liberated_meeple.push(meeple);
            }
        }


        // @todo implement scoring and meeple tracking in success result
        Ok(TilePlacementSuccess {
            liberated_meeple,
            score_delta
        })
    }

    /// Note this finds the score of the current board state; it ignores any previous score delta
    /// caused by meeple being liberated
    pub fn calculate_board_score(&self) -> Score {
        let mut score_delta = Score::new();

        for (_, connected_region) in &self.connected_regions {
            if let Some(winning_player) = connected_region.majority_meeple_player_id(self) {
                score_delta.add_score(winning_player, connected_region.score(self));
            }
        }

        score_delta

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
        tile: &PlacedTile,
        tile_connected_regions: Option<&Vec<ConnectedRegion>>,
    ) -> Result<(), InvalidTilePlacement> {

        if let Some((region_index, _)) = &tile.meeple {

            match tile.tile.regions.get(**region_index) {
                None => return Err(InvalidTilePlacement::InvalidMeeplePlacementIndex),
                Some(Region::Water {..}) => return Err(InvalidTilePlacement::MeepleCannotBePlacedInRiver),
                _ => ()
            }
        }

        // empty board is always valid for placement of a tile
        if self.placed_tiles.is_empty() {
            return Ok(());
        }

        if self.placed_tiles.contains_key(&tile.placement.coordinate) {
            return Err(InvalidTilePlacement::TileAlreadyAtCoordinate);
        }

        let surrounding_regions = self.get_surrounding_regions(&tile.placement.coordinate);

        if surrounding_regions.iter().all(|region| region.is_none()) {
            return Err(InvalidTilePlacement::TileDoesNotContactPlacedTiles);
        }

        let own_regions = tile.tile.list_oriented_region_types(tile.placement.rotations);

        let region_pairings: Vec<_> = own_regions.iter().zip(surrounding_regions).collect();

        if region_pairings.iter().any(|(own_region, neighbor_region)| match neighbor_region {
            Some(region) if &region != own_region => true,
            _ => false,
        })
        {
            return Err(InvalidTilePlacement::TileEdgesDoNotMatchPlacedTiles);
        }


        if own_regions.iter().any(|r| matches!(r, RegionType::Water)) {
            let paired_water = region_pairings.iter().filter(
                |(own_region, neighbor_region)| match (neighbor_region, own_region) {
                    (Some(RegionType::Water), RegionType::Water) => true,
                    _ => false,
                },
            ).count();

            if paired_water < 1 {
                return Err(InvalidTilePlacement::RiverMustBeConnected);
            }

            if tile.tile != &RIVER_TERMINATOR {
                let (_, prev_tile) = self.placed_tiles.last().expect("There should always be a last tile");

                let direction_to_prev = tile.placement.coordinate.direction_to_adjacent_coordinate(prev_tile.placement.coordinate);

                let current_heading = tile.get_opposite_river_end_direction(direction_to_prev);
                let previous_source = prev_tile.get_opposite_river_end_direction(direction_to_prev.compass_opposite());

                if previous_source == current_heading {
                    return Err(InvalidTilePlacement::RiverMustNotImmediatelyTurnOnItself);
                }
            }
        }

        if let Some((region_index, _)) = &tile.meeple {
            // avoid recomputing tile regions if we have already done so previously
            let tile_connected_regions = if let Some(tile_connected_regions) = tile_connected_regions {
                tile_connected_regions
            } else {
                &tile.own_connected_regions()
            };

            let meeple_connected_regions = tile_connected_regions.iter().filter(|r| {
                r.tile_regions.iter().any(|tr| tr.region_index == *region_index)
            });

            for connected_region in meeple_connected_regions {
                let regions_to_merge = self.get_candidate_regions_to_merge(connected_region);
                for region_id in regions_to_merge {
                    let joined_region = self.connected_regions.get(&region_id).expect("should exist");
                    if !joined_region.residents(self).is_empty() {
                        return Err(InvalidTilePlacement::OtherMeepleAlreadyInConnectedRegion);
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn list_adjacent_tiles(
        &self,
        board_coordinate: &BoardCoordinate,
    ) -> Vec<(CardinalDirection, Option<&PlacedTile>)> {
        board_coordinate.adjacent_coordinates().into_iter().map(|(direction, coordinate)| (direction, self.placed_tiles.get(&coordinate))).collect()
    }

    pub(crate) fn list_surrounding_tiles(
        &self,
        board_coordinate: &BoardCoordinate,
    ) -> Vec<&PlacedTile> {
        board_coordinate.surrounding_coordinates().into_iter().filter_map(|coordinate| self.placed_tiles.get(&coordinate)).collect()
    }

    fn get_surrounding_regions(
        &self,
        board_coordinate: &BoardCoordinate,
    ) -> Vec<Option<RegionType>> {
        self.list_adjacent_tiles(board_coordinate).iter().flat_map(|(direction, tile)| {
            if let Some(adjacent_tile) = tile {
                adjacent_tile.list_regions_on_edge(&direction.compass_opposite()).iter().map(|region_type| Some(region_type.clone())).collect()
            } else {
                vec![None; 3]
            }
        }).collect()
    }

    pub(crate) fn get_connected_regions(&self) -> Vec<&ConnectedRegion> {
        self.connected_regions.values().collect()
    }

    pub(crate) fn render(&self, style: &RenderStyle) -> String {
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
                    vec![" ".repeat(TILE_WIDTH * 2); TILE_WIDTH]
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
    use crate::tile::RegionType::{Field, Road};
    use crate::tile_definitions::{CLOISTER_IN_FIELD, CORNER_RIVER, CORNER_ROAD, STRAIGHT_RIVER, STRAIGHT_ROAD};

    #[test]
    fn test_valid_on_first_tile() {
        let board = Board::new();

        assert!(board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: None,
            },
            None,
        ).is_ok())
    }

    #[test]
    fn test_invalid_if_tile_already_at_coordinate() {
        let board = Board::new_with_tiles(vec![PlacedTile {
            tile: &STRAIGHT_ROAD,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
            meeple: None,
        }]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: None,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::TileAlreadyAtCoordinate)
        ))
    }

    #[test]
    fn test_invalid_if_tile_does_not_contact_placed_tiles() {
        let board = Board::new_with_tiles(vec![PlacedTile {
            tile: &STRAIGHT_ROAD,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
            meeple: None,
        }]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 2, y: 0 },
                    rotations: 0,
                },
                meeple: None,
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
        let board = Board::new_with_tiles(vec![
            PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: None,
            },
            PlacedTile {
                tile: &CORNER_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: -1 },
                    rotations: 0,
                },
                meeple: None,
            },
            PlacedTile {
                tile: &CORNER_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 1, y: -1 },
                    rotations: 1,
                },
                meeple: None,
            },
        ]).unwrap();

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
        let board = Board::new_with_tiles(vec![
            PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: None,
            },
            PlacedTile {
                tile: &CORNER_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: -1 },
                    rotations: 0,
                },
                meeple: None,
            },
            PlacedTile {
                tile: &CORNER_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 1, y: -1 },
                    rotations: 1,
                },
                meeple: None,
            },
        ]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 1, y: 0 },
                    rotations: 1,
                },
                meeple: None,
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
        let board = Board::new_with_tiles(vec![PlacedTile {
            tile: &STRAIGHT_RIVER,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
            meeple: None,
        }]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_RIVER,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 1, y: 0 },
                    rotations: 0,
                },
                meeple: None,
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
        let board = Board::new_with_tiles(vec![PlacedTile {
            tile: &CORNER_RIVER,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
            meeple: None,
        }]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &CORNER_RIVER,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: -1 },
                    rotations: 3,
                },
                meeple: None,
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::RiverMustNotImmediatelyTurnOnItself)
        ))
    }

    #[test]
    fn test_invalid_if_meeple_already_in_region() {
        let board = Board::new_with_tiles(vec![PlacedTile {
            tile: &STRAIGHT_ROAD,
            placement: TilePlacement {
                coordinate: BoardCoordinate { x: 0, y: 0 },
                rotations: 0,
            },
            meeple: Some((RegionIndex::new(0), Meeple::dummy())),
        }]).unwrap();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &STRAIGHT_ROAD,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: -1 },
                    rotations: 0,
                },
                meeple: Some((RegionIndex::new(0), Meeple::dummy())),
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::OtherMeepleAlreadyInConnectedRegion)
        ))
    }

    #[test]
    fn test_invalid_if_meeple_placed_in_invalid_region() {
        let board = Board::new();

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &RIVER_TERMINATOR,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: Some((RegionIndex::new(1) /* the river */, Meeple::dummy())),
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::MeepleCannotBePlacedInRiver)
        ));

        let res = board.validate_tile_placement(
            &PlacedTile {
                tile: &RIVER_TERMINATOR,
                placement: TilePlacement {
                    coordinate: BoardCoordinate { x: 0, y: 0 },
                    rotations: 0,
                },
                meeple: Some((RegionIndex::new(2) /* no such index */, Meeple::dummy())),
            },
            None,
        );

        assert!(matches!(
            res,
            Err(InvalidTilePlacement::InvalidMeeplePlacementIndex)
        ))
    }

    #[test]
    fn test_meeple_are_liberated_when_region_closes() {
        let mut board = Board::new_with_tiles(vec![
            PlacedTile::new_with_meeple(&CORNER_ROAD, -1, -1, 0, (RegionIndex::new(1) /* the outer field */, Meeple::dummy())),
            PlacedTile::new_with_meeple(&STRAIGHT_ROAD, -1, 0, 0, (RegionIndex::new(1) /* the inner field */, Meeple::dummy())),
            PlacedTile::new(&CORNER_ROAD, -1, 1, 3),
            PlacedTile::new(&STRAIGHT_ROAD, 0, -1, 1),
            PlacedTile::new_with_meeple(&CORNER_ROAD, 1, -1, 1, (RegionIndex::new(2) /* the road */, Meeple::dummy())),
            PlacedTile::new(&STRAIGHT_ROAD, 1, 0, 0),
            PlacedTile::new(&CORNER_ROAD, 1, 1, 2),
            PlacedTile::new_with_meeple(&CLOISTER_IN_FIELD, 0, 0, 0, (RegionIndex::new(1) /* the cloister */, Meeple::dummy())),
        ]).unwrap();

        println!("{}", board.render(RenderStyle::Ascii));

        let result = board.place_tile(PlacedTile::new(&STRAIGHT_ROAD, 0, 1, 1)).expect("should succeed");

        println!("{}", board.render(RenderStyle::Ascii));

        assert_eq!(result.liberated_meeple.len(), 3);
    }
}
