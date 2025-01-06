use crate::tile::CardinalDirection::{
    East, EastNorthEast, EastSouthEast, North, NorthNorthEast, NorthNorthWest, South,
    SouthSouthEast, SouthSouthWest, West, WestNorthWest, WestSouthWest,
};
use crate::tile::{Region, RenderCell, TileCoordinate, TileDefinition, TileRenderRepresentation};

// Definitions copied from https://cad.onshape.com/documents/04cfee738b84b4699685349a/w/f6c7a218fb2ae3244c5e18ee/e/e45463d6dd17036cc38b1be6

pub const CROSS_INTERSECTION: TileDefinition = TileDefinition {
    count: 1,
    name: "Cross intersection",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
RRRFRRR
FFFRFFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::Road {
            edges: &[South],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
        },
        Region::Road {
            edges: &[East],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
        Region::Road {
            edges: &[West],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Field {
            edges: &[EastSouthEast, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 4, y: 4 },
        },
        Region::Field {
            edges: &[SouthSouthWest, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 2, y: 4 },
        },
    ],
};
pub const THREE_WAY_JUNCTION_WITH_CITY: TileDefinition = TileDefinition {
    count: 3,
    name: "Three-way junction with city",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
RRRFRRR
FFFFFFF
FFCCCFF
+CCCCC+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::Road {
            edges: &[East],
            meeple_coordinate: TileCoordinate { x: 4, y: 3 },
        },
        Region::Road {
            edges: &[West],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::Field {
            edges: &[EastSouthEast, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 4 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
        Region::City {
            edges: &[SouthSouthEast, South, SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
            pennant: false,
        },
    ],
};
pub const STRAIGHT_CITY_WITH_SIDE_FIELDS: TileDefinition = TileDefinition {
    count: 1,
    name: "Straight city with side fields",
    render: ascii_to_tile(
        "
+CCCCC+
FCCCCCF
FFFCCFF
FFCCCFF
FFCCFFF
FCCCCCF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 4 },
            pennant: false,
        },
        Region::Field {
            edges: &[WestSouthWest, West, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[EastNorthEast, East, EastSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
    ],
};
pub const STRAIGHT_CITY_WITH_SIDE_FIELDS_AND_PENNANT: TileDefinition = TileDefinition {
    count: 2,
    name: "Straight city with side fields and pennant",
    render: ascii_to_tile(
        "
+CCCCC+
FCCCCCF
FFCCFFF
FFCPCFF
FFFCCFF
FCCCCCF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 2 },
            pennant: true,
        },
        Region::Field {
            edges: &[WestSouthWest, West, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[EastNorthEast, East, EastSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
    ],
};
pub const STRAIGHT_ROAD_WITH_SIDE_CITY: TileDefinition = TileDefinition {
    count: 4,
    name: "Straight road with side city",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFFFFF
RRRRRRR
FFFFFFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[EastNorthEast, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::Field {
            edges: &[
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 5 },
        },
        Region::Road {
            edges: &[East, West],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
    ],
};
pub const FIELD_WITH_TWO_CORNER_CITIES: TileDefinition = TileDefinition {
    count: 2,
    name: "Field with two corner cities",
    render: ascii_to_tile(
        "
+FFFFF+
FFFFFFC
FFFFFCC
FFFFFCC
FFFFFCC
FFCCCFC
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[EastNorthEast, East, EastSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
            pennant: false,
        },
        Region::City {
            edges: &[SouthSouthEast, South, SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
            pennant: false,
        },
        Region::Field {
            edges: &[
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
    ],
};
pub const CORNER_ROAD_WITH_PENNANTED_CORNER_CITY: TileDefinition = TileDefinition {
    count: 2,
    name: "Corner road with pennanted corner city",
    render: ascii_to_tile(
        "
+CCCCC+
FFFCPCC
FFFFCCC
RRFFFCC
FRRFFFC
FFRRFFC
+FFRFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 5, y: 2 },
            pennant: true,
        },
        Region::Field {
            edges: &[SouthSouthEast, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Field {
            edges: &[SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 5 },
        },
        Region::Road {
            edges: &[South, West],
            meeple_coordinate: TileCoordinate { x: 2, y: 4 },
        },
    ],
};
pub const THREE_WAY_JUNCTION: TileDefinition = TileDefinition {
    count: 4,
    name: "Three-way junction",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
RRRFRRR
FFFFFFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::Road {
            edges: &[East],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
        Region::Road {
            edges: &[West],
            meeple_coordinate: TileCoordinate { x: 1, y: 3 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 1 },
        },
        Region::Field {
            edges: &[
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 4 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
    ],
};
pub const RIVER_TERMINATOR: TileDefinition = TileDefinition {
    count: 2,
    name: "River terminator",
    render: ascii_to_tile(
        "
+FFFFF+
FFFFFFF
FFWWFFF
FFWWWWW
FFWWFFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Field {
            edges: &[
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 4, y: 1 },
        },
        Region::Water { edges: &[East] },
    ],
};
pub const CORNER_ROAD: TileDefinition = TileDefinition {
    count: 9,
    name: "Corner road",
    render: ascii_to_tile(
        "
+FFFFF+
FFFFFFF
FFFFFFF
FFFFRRR
FFFRRFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::Field {
            edges: &[EastSouthEast, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 5 },
        },
        Region::Field {
            edges: &[
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Road {
            edges: &[East, South],
            meeple_coordinate: TileCoordinate { x: 4, y: 3 },
        },
    ],
};
pub const RIVER_CORNER_WITH_ROAD_CORNER: TileDefinition = TileDefinition {
    count: 1,
    name: "River corner with road corner",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFRRFFF
RRRFFWW
FFFFWWF
FFFWWFF
+FFWFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North, West],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast, SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::Water {
            edges: &[East, South],
        },
        Region::Field {
            edges: &[EastSouthEast, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 5 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
    ],
};
pub const CORNER_CITY: TileDefinition = TileDefinition {
    count: 3,
    name: "Corner city",
    render: ascii_to_tile(
        "
+FFFFF+
CFFFFFF
CCFFFFF
CCFFFFF
CCCFFFF
CCCCFFF
+CCCCC+
",
    ),
    regions: &[
        Region::Field {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::City {
            edges: &[
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 5 },
            pennant: false,
        },
    ],
};
pub const SIDE_CITY_WITH_SIDE_ROAD_AND_PENNANT: TileDefinition = TileDefinition {
    count: 2,
    name: "Side city with side road and pennant",
    render: ascii_to_tile(
        "
+CCCCC+
CCCCCCC
CCCPCCC
CCCCCCC
CCCCCCC
CCFRFCC
+FFRFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 3 },
            pennant: true,
        },
        Region::Field {
            edges: &[SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 4, y: 5 },
        },
        Region::Road {
            edges: &[South],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
        },
        Region::Field {
            edges: &[SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 5 },
        },
    ],
};
pub const CORNER_ROAD_WITH_CORNER_CITY: TileDefinition = TileDefinition {
    count: 3,
    name: "Corner road with corner city",
    render: ascii_to_tile(
        "
+FFRFF+
FFRRFCC
FRRFFCC
RRFFCCC
FFFCCCC
FCCCCCC
+CCCCC+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North, West],
            meeple_coordinate: TileCoordinate { x: 2, y: 1 },
        },
        Region::Field {
            edges: &[NorthNorthEast, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::City {
            edges: &[
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 4, y: 5 },
            pennant: false,
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
    ],
};
pub const STRAIGHT_RIVER_WITH_TWO_SIDE_CITIES: TileDefinition = TileDefinition {
    count: 1,
    name: "Straight river with two side cities",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFFFFF
WWWWWWW
FFFFFFF
FFCCCFF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[EastNorthEast, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        },
        Region::Water {
            edges: &[East, West],
        },
        Region::Field {
            edges: &[EastSouthEast, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 4, y: 4 },
        },
        Region::City {
            edges: &[SouthSouthEast, South, SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
            pennant: false,
        },
    ],
};
pub const CORNER_ROAD_WITH_SIDE_CITY: TileDefinition = TileDefinition {
    count: 3,
    name: "Corner road with side city",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFFFFF
RRRFFFF
FFRRFFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 4, y: 3 },
        },
        Region::Road {
            edges: &[South, West],
            meeple_coordinate: TileCoordinate { x: 2, y: 3 },
        },
        Region::Field {
            edges: &[SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 5 },
        },
    ],
};
pub const CLOISTER_WITH_ROAD_AND_RIVER: TileDefinition = TileDefinition {
    count: 1,
    name: "Cloister with road and river",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
WWWRWWW
FFFTTFF
FFFTFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 2 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 4, y: 1 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 2, y: 1 },
        },
        Region::Water {
            edges: &[East, West],
        },
        Region::Field {
            edges: &[
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 1, y: 4 },
        },
        Region::Cloister {
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
        },
    ],
};
pub const CLOISTER_IN_FIELD: TileDefinition = TileDefinition {
    count: 4,
    name: "Cloister in field",
    render: ascii_to_tile(
        "
+FFFFF+
FFFFFFF
FFFFFFF
FFTTFFF
FFFTFFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Field {
            edges: &[
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::Cloister {
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
    ],
};
pub const THREE_SIDED_CITY: TileDefinition = TileDefinition {
    count: 3,
    name: "Three sided city",
    render: ascii_to_tile(
        "
+FFFFF+
CCFFFCC
CCCCCCC
CCCCCCC
CCCCCCC
CCCCCCC
+CCCCC+
",
    ),
    regions: &[
        Region::Field {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
        },
        Region::City {
            edges: &[
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 4 },
            pennant: false,
        },
    ],
};
pub const CENTRE_CITY_WITH_PENNANT: TileDefinition = TileDefinition {
    count: 1,
    name: "Centre city with pennant",
    render: ascii_to_tile(
        "
+CCCCC+
CCCCCCC
CCCCCCC
CCCPCCC
CCCCCCC
CCCCCCC
+CCCCC+
",
    ),
    regions: &[Region::City {
        edges: &[
            North,
            NorthNorthEast,
            EastNorthEast,
            East,
            EastSouthEast,
            SouthSouthEast,
            South,
            SouthSouthWest,
            WestSouthWest,
            West,
            WestNorthWest,
            NorthNorthWest,
        ],
        meeple_coordinate: TileCoordinate { x: 2, y: 2 },
        pennant: true,
    }],
};
pub const SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE: TileDefinition = TileDefinition {
    count: 1,
    name: "Side city with straight river and bridge",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFRFFF
WWWRWWW
FFFRFFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 2 },
        },
        Region::Water {
            edges: &[East, West],
        },
        Region::Field {
            edges: &[EastSouthEast, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 5 },
        },
        Region::Road {
            edges: &[South],
            meeple_coordinate: TileCoordinate { x: 3, y: 4 },
        },
        Region::Field {
            edges: &[SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 5 },
        },
        Region::Field {
            edges: &[WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 2 },
        },
    ],
};
pub const STRAIGHT_ROAD: TileDefinition = TileDefinition {
    count: 8,
    name: "Straight road",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
FFFRFFF
FFFRFFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North, South],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::Field {
            edges: &[
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 5, y: 2 },
        },
        Region::Field {
            edges: &[
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 1, y: 4 },
        },
    ],
};
pub const STRAIGHT_RIVER_WITH_STRAIGHT_ROAD_AND_BRIDGE: TileDefinition = TileDefinition {
    count: 1,
    name: "Straight river with straight road and bridge",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
WWWRWWW
FFFRFFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North, South],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::Field {
            edges: &[NorthNorthEast, EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 1 },
        },
        Region::Water {
            edges: &[East, West],
        },
        Region::Field {
            edges: &[EastSouthEast, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 5 },
        },
        Region::Field {
            edges: &[SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 5 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
    ],
};
pub const CORNER_CITY_WITH_PENNANT: TileDefinition = TileDefinition {
    count: 2,
    name: "Corner city with pennant",
    render: ascii_to_tile(
        "
+FFFFF+
CFFFFFF
CCFFFFF
CCCFFFF
CCCCFFF
CCPCCFF
+CCCCC+
",
    ),
    regions: &[
        Region::Field {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 4, y: 2 },
        },
        Region::City {
            edges: &[
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 1, y: 4 },
            pennant: true,
        },
    ],
};
pub const THREE_SIDED_CITY_WITH_PENNANT: TileDefinition = TileDefinition {
    count: 1,
    name: "Three sided city with pennant",
    render: ascii_to_tile(
        "
+CCCCC+
CCCCCCF
CCCCCFF
CCPCCFF
CCCCCFF
CCCCCCF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 2 },
            pennant: true,
        },
        Region::Field {
            edges: &[EastNorthEast, East, EastSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
    ],
};
pub const CORNER_CITY_WITH_CORNER_RIVER: TileDefinition = TileDefinition {
    count: 1,
    name: "Corner city with corner river",
    render: ascii_to_tile(
        "
+CCCCC+
FFFCCCC
FFFFCCC
WWFFFCC
FWWFFFC
FFWWFFC
+FFWFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 5, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[SouthSouthEast, WestNorthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::Field {
            edges: &[SouthSouthWest, WestSouthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 5 },
        },
        Region::Water {
            edges: &[South, West],
        },
    ],
};
pub const STRAIGHT_RIVER: TileDefinition = TileDefinition {
    count: 2,
    name: "Straight river",
    render: ascii_to_tile(
        "
+FFWFF+
FFWWFFF
FFWFFFF
FFWWWFF
FFFFWFF
FFFWWFF
+FFWFF+
",
    ),
    regions: &[
        Region::Water {
            edges: &[North, South],
        },
        Region::Field {
            edges: &[
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 5, y: 2 },
        },
        Region::Field {
            edges: &[
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 1, y: 4 },
        },
    ],
};
pub const CORNER_RIVER: TileDefinition = TileDefinition {
    count: 2,
    name: "Corner river",
    render: ascii_to_tile(
        "
+FFWFF+
FFFWWFF
FFFFWFF
WWWFWFF
FFWWWFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Water {
            edges: &[North, West],
        },
        Region::Field {
            edges: &[
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
        },
        Region::Field {
            edges: &[WestNorthWest, NorthNorthWest],
            meeple_coordinate: TileCoordinate { x: 1, y: 1 },
        },
    ],
};
pub const SIDE_CITY: TileDefinition = TileDefinition {
    count: 5,
    name: "Side city",
    render: ascii_to_tile(
        "
+FFFFF+
FFFFFFF
FFFFFFF
FFFFFFF
FFFFFFF
FFCCCFF
+CCCCC+
",
    ),
    regions: &[
        Region::Field {
            edges: &[
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 2 },
        },
        Region::City {
            edges: &[SouthSouthEast, South, SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
            pennant: false,
        },
    ],
};
pub const OPPOSING_SIDE_CITIES: TileDefinition = TileDefinition {
    count: 3,
    name: "Opposing side cities",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFFFFF
FFFFFFF
FFFFFFF
FFCCCFF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[
                EastNorthEast,
                East,
                EastSouthEast,
                WestSouthWest,
                West,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
        Region::City {
            edges: &[SouthSouthEast, South, SouthSouthWest],
            meeple_coordinate: TileCoordinate { x: 3, y: 5 },
            pennant: false,
        },
    ],
};
pub const THREE_SIDED_CITY_WITH_ROAD: TileDefinition = TileDefinition {
    count: 1,
    name: "Three sided city with road",
    render: ascii_to_tile(
        "
+CCCCC+
CCCCCCF
CCCCCFF
CCCCCRR
CCCCCFF
CCCCCCF
+CCCCC+
",
    ),
    regions: &[
        Region::City {
            edges: &[
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
                North,
                NorthNorthEast,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 3 },
            pennant: false,
        },
        Region::Field {
            edges: &[EastNorthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 2 },
        },
        Region::Road {
            edges: &[East],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
        Region::Field {
            edges: &[EastSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 4 },
        },
    ],
};
pub const STRAIGHT_ROAD_WITH_RIGHT_CORNER_ROAD: TileDefinition = TileDefinition {
    count: 3,
    name: "Straight road with right corner road",
    render: ascii_to_tile(
        "
+CCCCC+
FFCCCFF
FFFFFFF
FFFFRRR
FFFRRFF
FFFRFFF
+FFRFF+
",
    ),
    regions: &[
        Region::City {
            edges: &[NorthNorthWest, North, NorthNorthEast],
            meeple_coordinate: TileCoordinate { x: 3, y: 1 },
            pennant: false,
        },
        Region::Field {
            edges: &[
                EastNorthEast,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 2, y: 3 },
        },
        Region::Road {
            edges: &[East, South],
            meeple_coordinate: TileCoordinate { x: 4, y: 4 },
        },
        Region::Field {
            edges: &[EastSouthEast, SouthSouthEast],
            meeple_coordinate: TileCoordinate { x: 5, y: 5 },
        },
    ],
};
pub const CLOISTER_WITH_ROAD: TileDefinition = TileDefinition {
    count: 2,
    name: "Cloister with road",
    render: ascii_to_tile(
        "
+FFRFF+
FFFRFFF
FFFRFFF
FFTTFFF
FFFTFFF
FFFFFFF
+FFFFF+
",
    ),
    regions: &[
        Region::Road {
            edges: &[North],
            meeple_coordinate: TileCoordinate { x: 3, y: 2 },
        },
        Region::Field {
            edges: &[
                NorthNorthEast,
                EastNorthEast,
                East,
                EastSouthEast,
                SouthSouthEast,
                South,
                SouthSouthWest,
                WestSouthWest,
                West,
                WestNorthWest,
                NorthNorthWest,
            ],
            meeple_coordinate: TileCoordinate { x: 5, y: 3 },
        },
        Region::Cloister {
            meeple_coordinate: TileCoordinate { x: 3, y: 3 },
        },
    ],
};
pub const ALL_TILE_DEFINITIONS: [TileDefinition; 33] = [
    CROSS_INTERSECTION,
    THREE_WAY_JUNCTION_WITH_CITY,
    STRAIGHT_CITY_WITH_SIDE_FIELDS,
    STRAIGHT_CITY_WITH_SIDE_FIELDS_AND_PENNANT,
    STRAIGHT_ROAD_WITH_SIDE_CITY,
    FIELD_WITH_TWO_CORNER_CITIES,
    CORNER_ROAD_WITH_PENNANTED_CORNER_CITY,
    THREE_WAY_JUNCTION,
    RIVER_TERMINATOR,
    CORNER_ROAD,
    RIVER_CORNER_WITH_ROAD_CORNER,
    CORNER_CITY,
    SIDE_CITY_WITH_SIDE_ROAD_AND_PENNANT,
    CORNER_ROAD_WITH_CORNER_CITY,
    STRAIGHT_RIVER_WITH_TWO_SIDE_CITIES,
    CORNER_ROAD_WITH_SIDE_CITY,
    CLOISTER_WITH_ROAD_AND_RIVER,
    CLOISTER_IN_FIELD,
    THREE_SIDED_CITY,
    CENTRE_CITY_WITH_PENNANT,
    SIDE_CITY_WITH_STRAIGHT_RIVER_AND_BRIDGE,
    STRAIGHT_ROAD,
    STRAIGHT_RIVER_WITH_STRAIGHT_ROAD_AND_BRIDGE,
    CORNER_CITY_WITH_PENNANT,
    THREE_SIDED_CITY_WITH_PENNANT,
    CORNER_CITY_WITH_CORNER_RIVER,
    STRAIGHT_RIVER,
    CORNER_RIVER,
    SIDE_CITY,
    OPPOSING_SIDE_CITIES,
    THREE_SIDED_CITY_WITH_ROAD,
    STRAIGHT_ROAD_WITH_RIGHT_CORNER_ROAD,
    CLOISTER_WITH_ROAD,
];

const fn ascii_to_tile(ascii: &'static str) -> TileRenderRepresentation {
    let mut repr: [[RenderCell; 7]; 7] = [[RenderCell::Corner; 7]; 7];

    // @todo once iterators are allowed in const fns, the following can be greatly simplified to
    // the following approach
    // for (row, line) in ascii.trim().lines().enumerate() {
    //     for (column, cell) in line.chars().enumerate() {
    //         let value = match cell {
    //             'F' => RenderCell::Field,
    //             'R' => RenderCell::Road,
    //             'C' => RenderCell::City,
    //             'T' => RenderCell::Cloister,
    //             v => panic!("Unexpected cell representation [{}] at [{}, {}]", v, column, row)
    //         };
    //
    //         repr[row][column] = value;
    //     }
    // }

    let mut i = 0;
    let mut row = 0;
    let mut column = 0;

    while i < ascii.len() {
        let byte = ascii.as_bytes()[i];

        if byte < 128 {
            i += 1;
        } else {
            panic!("Invalid UTF-8 sequence")
        }

        if byte == b'\n' {
            if column == 0 {
                continue;
            }
            column = 0;
            row += 1;
            continue;
        }

        if column == 7 {
            panic!("Count of characters in row exceeds expected 7");
        }

        let value = match byte {
            b'F' => RenderCell::Field,
            b'R' => RenderCell::Road,
            b'C' => RenderCell::City,
            b'T' => RenderCell::Cloister,
            b'P' => RenderCell::Pennant,
            b'W' => RenderCell::Water,
            b'+' => RenderCell::Corner, // @todo remove line once replaced all corners
            _ => panic!("Unexpected cell representation"),
        };

        repr[row][column] = value;

        column += 1;
    }

    TileRenderRepresentation(repr)
}
