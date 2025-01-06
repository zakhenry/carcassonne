// data class UniqueTileRegion(val tileId: UUID, val regionIndex: Int, val region: Region, val edgeless: Boolean)

use std::collections::HashSet;
use std::rc::Rc;
use uuid::Uuid;
use crate::player::Meeple;
use crate::tile::{Region, RegionType};

#[derive(Debug)]
pub(crate)  struct UniqueTileRegion {
    tile_id: usize,
    region_index: usize,
    region: &'static Region,
    edgeless: bool
}

#[derive(Debug)]
pub(crate) struct ConnectedRegion {
    region: RegionType,
    connected_region_edges: HashSet<UniqueTileRegion>,
    residents: HashSet<Rc<Meeple>>,
    id: Uuid,
}
