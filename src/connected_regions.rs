// data class UniqueTileRegion(val tileId: UUID, val regionIndex: Int, val region: Region, val edgeless: Boolean)

use crate::board::Board;
use crate::player::{Meeple, PlayerId, RegionIndex};
use crate::tile::{
    BoardCoordinate, CardinalDirection, PlacedTile, Region, RegionType,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct PlacedTileEdge {
    pub(crate) coordinate: BoardCoordinate,
    pub(crate) global_direction: CardinalDirection,
}

impl PlacedTileEdge {
    pub(crate) fn opposing_tile_edge(&self) -> PlacedTileEdge {
        PlacedTileEdge {
            coordinate: self
                .coordinate
                .adjacent_in_direction(&self.global_direction),
            global_direction: self.global_direction.tile_opposite(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PlacedTileRegion {
    pub(crate) tile_position: BoardCoordinate, // this is used to look up the placed tile in the board index
    pub(crate) region_index: RegionIndex,
    pub(crate) region: &'static Region,
}

impl PlacedTileRegion {
    pub(crate) fn new(placed_tile: &PlacedTile, region_index: RegionIndex) -> Self {
        let region = &placed_tile.tile.regions[*region_index];

        Self {
            tile_position: placed_tile.placement.coordinate,
            region_index,
            region,
        }
    }

    pub(crate) fn edgeless(&self) -> bool {
        self.region.edges().is_empty()
    }
}

pub(crate) type ConnectedRegionId = Uuid;

#[derive(Debug)]
pub(crate) struct ConnectedRegion {
    pub(crate) id: ConnectedRegionId,
    pub(crate) region_type: RegionType,
    pub(crate) tile_regions: Vec<PlacedTileRegion>,
    pub(crate) adjacent_regions: HashSet<ConnectedRegionId>,
    pub(crate) connected_edges: HashMap<PlacedTileEdge, Option<PlacedTileEdge>>,
}

#[derive(Debug)]
pub(crate) enum ConnectedRegionMergeFailure {
    RegionTypeMismatch,
    EmptyCollection,
}


impl ConnectedRegion {
    pub(crate) fn merge_mut(
        &mut self,
        other: Self,
    ) -> Result<&mut Self, ConnectedRegionMergeFailure> {
        if self.region_type != other.region_type {
            return Err(ConnectedRegionMergeFailure::RegionTypeMismatch);
        }

        self.tile_regions.extend(other.tile_regions);
        self.adjacent_regions.extend(other.adjacent_regions);

        for (own_edge, foreign_edge) in other.connected_edges {
            let opposite = own_edge.opposing_tile_edge();

            if self.connected_edges.contains_key(&opposite) {
                self.connected_edges.insert(opposite, Some(own_edge));
            } else {
                self.connected_edges.insert(own_edge, foreign_edge);
            }
        }

        Ok(self)
    }

    pub(crate) fn residents<'a>(&self, board: &'a Board) -> Vec<(&'a PlacedTile, &'a RegionIndex, &'a Meeple)> {
        self.tile_regions
            .iter()
            .map(|r| {
                let tile = board
                    .get_tile_at_coordinate(&r.tile_position)
                    .expect("should exist");

                (tile, r.region_index)
            })
            .filter_map(|(tile, region_index)| {
                match &tile.meeple {
                    Some((meeple_region_index, meeple)) if &region_index == meeple_region_index => {
                        Some((tile, meeple_region_index, meeple))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    pub(crate) fn is_closed(&self) -> bool {
        !self.connected_edges.is_empty() && self.connected_edges.values().all(|e| e.is_some())
    }

}

trait ConnectedRegionCollection {
    fn merge_all(self) -> Result<ConnectedRegion, ConnectedRegionMergeFailure>;
}

impl ConnectedRegionCollection for Vec<ConnectedRegion> {
    fn merge_all(self) -> Result<ConnectedRegion, ConnectedRegionMergeFailure> {
        let mut iter = self.into_iter();

        let mut acc = match iter.next() {
            Some(first) => first,
            None => return Err(ConnectedRegionMergeFailure::EmptyCollection),
        };

        // Fold the remaining items
        for region in iter {
            acc.merge_mut(region)?;
        }

        Ok(acc)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::connected_regions::{
        ConnectedRegion, ConnectedRegionCollection, ConnectedRegionMergeFailure, PlacedTileEdge
        ,
    };
    use crate::tile::CardinalDirection::{EastSouthEast, WestSouthWest};
    use crate::tile::RegionType::{City, Cloister, Field, Road};
    use crate::tile::{BoardCoordinate, PlacedTile, RegionType, RenderStyle};
    use crate::tile_definitions::{
        CLOISTER_IN_FIELD, CORNER_ROAD, CROSS_INTERSECTION, STRAIGHT_ROAD, THREE_SIDED_CITY,
    };
    use uuid::Uuid;

    #[test]
    fn should_derive_adjacent_regions() {
        let regions = PlacedTile::new(&STRAIGHT_ROAD, 0, 0, 0).own_connected_regions();

        assert_eq!(regions.len(), 3);

        let road_region = regions
            .iter()
            .find(|r| r.region_type == Road)
            .expect("should exist");
        assert_eq!(road_region.adjacent_regions.len(), 2);

        let field_regions: Vec<_> = regions.iter().filter(|r| r.region_type == Field).collect();
        assert_eq!(field_regions.len(), 2);

        for field_region in field_regions {
            assert_eq!(field_region.adjacent_regions.len(), 1);
        }
    }

    #[test]
    fn opposing_tile_edge_should_give_adjacent_edge_position() {
        let edge = PlacedTileEdge {
            coordinate: BoardCoordinate::new(0, 0),
            global_direction: EastSouthEast,
        };

        assert_eq!(
            edge.opposing_tile_edge(),
            PlacedTileEdge {
                coordinate: BoardCoordinate::new(1, 0),
                global_direction: WestSouthWest
            }
        )
    }

    #[test]
    fn should_error_when_merging_mismatched_regions() {
        let mut test_region = ConnectedRegion {
            region_type: City,
            tile_regions: vec![],
            id: Uuid::new_v4(),
            adjacent_regions: Default::default(),
            connected_edges: Default::default(),
        };

        let merge_result = test_region.merge_mut(ConnectedRegion {
            region_type: Field,
            tile_regions: vec![],
            id: Uuid::new_v4(),
            adjacent_regions: Default::default(),
            connected_edges: Default::default(),
        });

        assert!(matches!(
            merge_result,
            Err(ConnectedRegionMergeFailure::RegionTypeMismatch)
        ))
    }

    #[test]
    fn should_merge_connected_regions() {
        let mut test_region = PlacedTile::new(&CLOISTER_IN_FIELD, 0, 0, 0)
            .own_connected_regions()
            .into_iter()
            .find(|r| r.region_type == Field)
            .expect("should exist");

        let other_region = PlacedTile::new(&THREE_SIDED_CITY, 0, 1, 0)
            .own_connected_regions()
            .into_iter()
            .find(|r| r.region_type == Field)
            .expect("should exist");

        let merge_result = test_region
            .merge_mut(other_region)
            .expect("should merge ok");

        assert_eq!(merge_result.tile_regions.len(), 2)
    }

    #[test]
    fn should_merge_connected_region_collection() {
        let mut collection: Vec<_> = PlacedTile::new(&STRAIGHT_ROAD, 0, 0, 0)
            .own_connected_regions()
            .into_iter()
            .filter(|r| r.region_type == Field)
            .collect();

        collection.extend(
            PlacedTile::new(&STRAIGHT_ROAD, 1, 0, 0)
                .own_connected_regions()
                .into_iter()
                .filter(|r| r.region_type == Field),
        );

        let merge_result = collection.merge_all().expect("should succeed");

        assert_eq!(merge_result.tile_regions.len(), 4)
    }

    fn assert_connected_regions(
        placed_tiles: &[PlacedTile],
        expectation: &[(RegionType, bool)],
    ) {
        let board = Board::new_with_tiles(placed_tiles.to_vec()).unwrap();

        println!("{}", board.render(&RenderStyle::Ascii));

        let connected_regions = board.get_connected_regions();

        // for region in &connected_regions {
        //     println!("{:?} {}", region.region_type, region.is_closed());
        // }

        let mut region_types: Vec<_> = connected_regions
            .into_iter()
            .map(|r| (r.region_type.clone(), r.is_closed()))
            .collect();

        let mut test = Vec::from_iter(expectation.iter().cloned());

        test.sort();
        region_types.sort();

        assert_eq!(test, region_types)
    }

    #[test]
    fn should_join_regions_into_a_single_region() {
        assert_connected_regions(
            &[
                PlacedTile::new(&CLOISTER_IN_FIELD, 0, 0, 0),
                PlacedTile::new(&CLOISTER_IN_FIELD, 1, 0, 0),
                PlacedTile::new(&CLOISTER_IN_FIELD, 1, 1, 0),
            ],
            &[
                (Cloister, false),
                (Cloister, false),
                (Cloister, false),
                (Field, false),
            ],
        )
    }

    #[test]
    fn should_keep_regions_divided_by_other_elements_distinct() {
        assert_connected_regions(
            &[
                PlacedTile::new(&STRAIGHT_ROAD, 0, 0, 0),
                PlacedTile::new(&STRAIGHT_ROAD, 0, 1, 0),
            ],
            &[(Field, false), (Field, false), (Road, false)],
        )
    }

    #[test]
    fn should_wrap_around_and_connect_regions() {
        assert_connected_regions(
            &[
                PlacedTile::new(&CORNER_ROAD, -1, -1, 0),
                PlacedTile::new(&STRAIGHT_ROAD, -1, 0, 0),
                PlacedTile::new(&CORNER_ROAD, -1, 1, 3),
                PlacedTile::new(&STRAIGHT_ROAD, 0, -1, 1),
                PlacedTile::new(&CORNER_ROAD, 1, -1, 1),
                PlacedTile::new(&STRAIGHT_ROAD, 1, 0, 0),
                PlacedTile::new(&CORNER_ROAD, 1, 1, 2),
                PlacedTile::new(&STRAIGHT_ROAD, 0, 1, 1),
                PlacedTile::new(&CLOISTER_IN_FIELD, 0, 0, 0),
            ],
            &[
                (Cloister, false),
                (Field, true),
                (Field, false),
                (Road, true),
            ],
        )
    }

    #[test]
    fn should_multi_merge_regions() {
        assert_connected_regions(
            &[
                PlacedTile::new(&CORNER_ROAD, -1, -1, 0),
                PlacedTile::new(&CROSS_INTERSECTION, -1, 0, 0),
                PlacedTile::new(&CORNER_ROAD, -1, 1, 3),
                PlacedTile::new(&STRAIGHT_ROAD, 0, -1, 1),
                PlacedTile::new(&CORNER_ROAD, 1, -1, 1),
                PlacedTile::new(&STRAIGHT_ROAD, 1, 0, 0),
                PlacedTile::new(&CORNER_ROAD, 1, 1, 2),
                PlacedTile::new(&STRAIGHT_ROAD, 0, 1, 1),
            ],
            &[
                (Field, false),
                (Field, false),
                (Road, false),
                (Road, false),
                (Road, true),
            ],
        )
    }
}
