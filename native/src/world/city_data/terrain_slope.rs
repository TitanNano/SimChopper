use crate::util::logger;

/// Describes the raised corners of a tile
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TerrainSlope {
    /// A tile that has no slope set yet
    Undetermined,

    /// Plane
    None,

    /// North -> South Slope
    South,

    /// South -> North Slope
    North,

    /// East -> West Slope
    West,

    /// West -> East Slope
    East,

    /// North-East + South -> North-West
    NorthWest,

    /// North-West + South -> North-East Slope
    NorthEast,

    /// South-East + North -> South-West
    SouthWest,

    /// South-West + North -> South-East
    SouthEast,

    /// North-West -> South + North-East Slope
    SouthNorthEast,

    /// north-east -> South + North-West Slope
    SouthNorthWest,

    /// South-East -> North + South-West Slope
    NorthSouthWest,

    /// South-West -> North + South-East Slope
    NorthSouthEast,

    /// Raised Plane
    All,

    // vertical for waterfall
    VertialCliff,

    // ----- Non standard tile slopes to patch holes in exisiting maps -----
    //
    /// South-West -> North-West + North-East * 2 + South-East
    NorthWestEast2SouthEast,

    /// South-East -> North-East + North-West * 2 + South-East
    NorthEastWest2SouthWest,

    /// North-West -> South-West + South-East * 2 + North-East
    SouthWestEast2NorthEast,

    /// North-East -> South-East + South-West * 2 + North-West
    SouthEastWest2NorthWest,

    /// North-West -> South * 2 + North-East Slope
    South2NorthEast,

    /// South-West -> East * 2 + North-West
    East2NorthWest,

    /// South-East -> North * 2 + South-West Slope
    North2SouthWest,

    /// North-East -> West * 2 + South-East
    West2SouthEast,

    /// North-East -> South * 2 + North-West Slope
    South2NorthWest,

    /// North-West -> East * 2 + South-West
    East2SouthWest,

    /// South-West -> North * 2 + South-East Slope
    North2SouthEast,

    /// South-East -> West * 2 + North-East
    West2NorthEast,

    /// North-East -> West + South-East * 2
    WestSouthEast2,

    /// North-West -> South + North-East * 2
    SouthNorthEast2,

    /// South-West -> East + North-West * 2
    EastNorthWest2,

    /// South-East -> North + South-West * 2
    NorthSouthWest2,

    /// South -> North-West * 2 + North-East
    NorthWest2East,

    /// South-East + North-East-> South-West * 2 + North-West
    SouthWest2NorthWest,

    /// North -> South-West + South-East * 2
    SouthEast2West,

    /// North-West + South-West -> North-East * 2 + South-East
    NorthEast2SouthEast,
    /// South-East -> West + North-East * 2
    WestNorthEast2,

    /// North-East -> South + North-West * 2
    SouthNorthWest2,

    /// North-West -> East + South-West * 2
    EastSouthWest2,

    /// South-West -> North + South-East * 2
    NorthSouthEast2,
}

impl TerrainSlope {
    pub const fn valid_north_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            // North is not raised at all
            Self::None | Self::South | Self::SouthWest | Self::SouthEast => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthWest2East, 0),
                // Lower Elevation
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::All, -1),
            ],

            // North is raised
            Self::North | Self::NorthSouthWest | Self::NorthSouthEast | Self::All => &[
                (Self::All, 0),
                (Self::South, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::South2NorthEast, -1),
            ],

            // North is raised in the West
            Self::West | Self::NorthWest | Self::SouthNorthWest | Self::SouthEastWest2NorthWest => {
                &[
                    (Self::SouthWest, 0),
                    (Self::West, 0),
                    (Self::NorthSouthWest, 0),
                    (Self::NorthEastWest2SouthWest, 0),
                    (Self::WestNorthEast2, 0),
                    // Lower Elevation
                    (Self::SouthEastWest2NorthWest, -1),
                    (Self::West2SouthEast, -1),
                ]
            }

            // North is raised in the East
            Self::East | Self::NorthEast | Self::SouthNorthEast | Self::SouthWestEast2NorthEast => {
                &[
                    (Self::SouthEast, 0),
                    (Self::East, 0),
                    (Self::NorthSouthEast, 0),
                    (Self::NorthWestEast2SouthEast, 0),
                    // Lower Elevation
                    (Self::SouthWestEast2NorthEast, -1),
                    (Self::WestSouthEast2, -1),
                ]
            }

            // North is raised in the west and 2x in the east
            Self::NorthWestEast2SouthEast => &[
                (Self::SouthEast, 1),
                (Self::NorthSouthEast, 1),
                (Self::NorthWestEast2SouthEast, 1),
                (Self::East, 1),
            ],

            // North is raised in the east and 2x in the west
            Self::NorthEastWest2SouthWest => &[
                (Self::SouthWest, 1),
                (Self::NorthSouthWest, 1),
                (Self::NorthEastWest2SouthWest, 1),
                (Self::West, 1),
            ],

            Self::South2NorthEast => &[(Self::SouthEast, 0)],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[(Self::SouthWest2NorthWest, 0)],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[(Self::SouthWest, 0)],

            Self::NorthWest2East => &[],
            Self::SouthWest2NorthWest => &[],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],

            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::All, 0),
                (Self::SouthWest2NorthWest, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::SouthWestEast2NorthEast, -1),
                (Self::SouthEastWest2NorthWest, -1),
            ],
        }
    }

    pub const fn valid_north_west_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::All, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::NorthWestEast2SouthEast, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::NorthSouthWest, 1),
                (Self::West, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::SouthEast, -1),
                (Self::NorthSouthEast, -1),
                (Self::East, -1),
                (Self::SouthEastWest2NorthWest, -1),
                (Self::SouthWestEast2NorthEast, -1),
                // Lower -2 Elevation
                (Self::SouthWestEast2NorthEast, -2),
            ],
            // Terrain is NOT raised in the North-West
            Self::None
            | Self::South
            | Self::SouthWest
            | Self::SouthEast
            | Self::SouthNorthEast
            | Self::SouthWestEast2NorthEast
            | Self::NorthEast
            | Self::East => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthWest2East, 0),
                (Self::NorthSouthWest, 0),
                (Self::West, 0),
                (Self::SouthWest, 0),
                (Self::NorthEastWest2SouthWest, 0),
                (Self::WestNorthEast2, 0),
                (Self::SouthWest2NorthWest, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::SouthEast, -1),
                (Self::NorthSouthEast, -1),
                (Self::East, -1),
                (Self::NorthWestEast2SouthEast, -1),
                (Self::SouthEastWest2NorthWest, -1),
                // Lower Elevation -2
                (Self::SouthWestEast2NorthEast, -2),
                (Self::South2NorthEast, -2),
            ],

            // Terrain is raised in the North-West
            Self::All
            | Self::North
            | Self::West
            | Self::NorthWest
            | Self::NorthWestEast2SouthEast
            | Self::NorthSouthWest
            | Self::NorthSouthEast
            | Self::SouthNorthWest
            | Self::SouthEastWest2NorthWest => &[
                (Self::All, 0),
                (Self::East, 0),
                (Self::South, 0),
                (Self::SouthEast, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::NorthWestEast2SouthEast, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::West2SouthEast, 0),
                //Higher Elevation
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::NorthSouthWest, 1),
                (Self::SouthWest, 1),
                (Self::West, 1),
                (Self::NorthEastWest2SouthWest, 1),
                // Lower Elevation
                (Self::WestSouthEast2, -1),
                (Self::SouthWestEast2NorthEast, -1),
            ],

            // Terrain is raised in the North-West x2
            Self::NorthEastWest2SouthWest => &[
                (Self::All, 1),
                (Self::None, 2),
                (Self::North, 2),
                (Self::NorthWest, 2),
                (Self::NorthEast, 2),
                (Self::NorthSouthWest, 2),
                (Self::SouthWest, 2),
                (Self::West, 2),
                (Self::NorthEastWest2SouthWest, 2),
            ],

            Self::South2NorthEast => &[(Self::SouthWest, 0)],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[(Self::South, 0)],

            Self::NorthWest2East => &[],
            Self::SouthWest2NorthWest => &[(Self::All, 0)],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],
        }
    }

    pub const fn valid_north_east_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::NorthEastWest2SouthWest, 0),
                (Self::All, 0),
                (Self::SouthWestEast2NorthEast, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::NorthSouthEast, 1),
                (Self::East, 1),
                (Self::SouthEast, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::SouthWest, -1),
                (Self::NorthSouthWest, -1),
                (Self::West, -1),
                (Self::SouthEastWest2NorthWest, -1),
                (Self::SouthWestEast2NorthEast, -1),
                // Lower Elevation -2
                (Self::SouthEastWest2NorthWest, -2),
                (Self::SouthWestEast2NorthEast, -2),
            ],

            // Terrain is NOT raised in the North-East
            Self::None
            | Self::West
            | Self::South
            | Self::SouthNorthWest
            | Self::NorthWest
            | Self::SouthWest
            | Self::SouthEast
            | Self::SouthEastWest2NorthWest => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthWest2East, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthEast, 0),
                (Self::East, 0),
                (Self::NorthWestEast2SouthEast, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::South, -1),
                (Self::SouthNorthWest, -1),
                (Self::SouthNorthEast, -1),
                (Self::SouthWest, -1),
                (Self::NorthSouthWest, -1),
                (Self::West, -1),
                (Self::NorthEastWest2SouthWest, -1),
                (Self::SouthWestEast2NorthEast, -1),
                (Self::WestSouthEast2, -1),
                (Self::WestNorthEast2, -1),
                // Lower elevation -2
                (Self::SouthEastWest2NorthWest, -2),
                (Self::West2SouthEast, -2),
            ],

            // Terrain is raised in the North-East
            Self::All
            | Self::East
            | Self::North
            | Self::NorthEast
            | Self::NorthSouthWest
            | Self::NorthSouthEast
            | Self::SouthNorthEast
            | Self::NorthEastWest2SouthWest
            | Self::SouthWestEast2NorthEast => &[
                (Self::All, 0),
                (Self::West, 0),
                (Self::South, 0),
                (Self::SouthWest, 0),
                (Self::NorthSouthWest, 0),
                (Self::SouthNorthWest, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::NorthEastWest2SouthWest, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::NorthSouthEast, 1),
                (Self::SouthEast, 1),
                (Self::East, 1),
                (Self::NorthWestEast2SouthEast, 1),
                // Lower elevation
                (Self::SouthEastWest2NorthWest, -1),
                (Self::South2NorthEast, -1),
                (Self::SouthWest2NorthWest, -1),
            ],

            // Terrain is raised in the North-East 2x
            Self::NorthWestEast2SouthEast => &[
                // Higher Elevation
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthWestEast2NorthEast, 1),
                (Self::All, 1),
                (Self::West, 1),
                (Self::NorthSouthWest, 1),
                (Self::NorthEastWest2SouthWest, 1),
                // Higher Elevation 2x
                (Self::None, 2),
                (Self::SouthEast, 2),
                (Self::NorthSouthEast, 2),
                (Self::NorthWest, 2),
                (Self::NorthEast, 2),
                (Self::North, 2),
                (Self::NorthWestEast2SouthEast, 2),
                (Self::East, 2),
            ],

            Self::South2NorthEast => &[(Self::South, 0)],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[
                // Lower Elevation
                (Self::SouthNorthWest, -1),
            ],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[(Self::SouthEast, 0)],

            Self::NorthWest2East => &[],
            Self::SouthWest2NorthWest => &[
                // Lower Elevation
                (Self::SouthNorthWest, -1),
                (Self::West, -1),
            ],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],
        }
    }

    pub const fn valid_south_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            // South is not raised at all
            Self::None | Self::North | Self::NorthWest | Self::NorthEast => &[
                (Self::None, 0),
                (Self::South, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::All, -1),
                (Self::North, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
            ],

            // South is raised
            Self::South | Self::SouthNorthEast | Self::SouthNorthWest | Self::All => &[
                (Self::All, 0),
                (Self::North, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::None, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
            ],

            // South is raised in the West
            Self::West | Self::SouthWest | Self::NorthSouthWest | Self::NorthEastWest2SouthWest => {
                &[
                    (Self::NorthWest, 0),
                    (Self::West, 0),
                    (Self::SouthNorthWest, 0),
                    (Self::SouthEastWest2NorthWest, 0),
                    (Self::WestSouthEast2, 0),
                    (Self::SouthWest2NorthWest, 0),
                    // Lower Elevation
                    (Self::NorthEastWest2SouthWest, -1),
                    (Self::NorthWest2East, -1),
                ]
            }

            // South is raised in the East
            Self::East | Self::SouthEast | Self::NorthSouthEast | Self::NorthWestEast2SouthEast => {
                &[
                    (Self::NorthEast, 0),
                    (Self::East, 0),
                    (Self::SouthNorthEast, 0),
                    (Self::SouthWestEast2NorthEast, 0),
                    (Self::South2NorthEast, 0),
                    // Lower Elevation
                    (Self::NorthWestEast2SouthEast, -1),
                    (Self::WestNorthEast2, -1),
                ]
            }

            // South is raised in the East 2x
            Self::SouthWestEast2NorthEast => &[
                (Self::NorthEast, 1),
                (Self::East, 1),
                (Self::SouthNorthEast, 1),
                (Self::SouthWestEast2NorthEast, 1),
            ],

            // South is raised in the West 2x
            Self::SouthEastWest2NorthWest => &[
                (Self::NorthWest, 1),
                (Self::West, 1),
                (Self::SouthNorthWest, 1),
                (Self::SouthEastWest2NorthWest, 1),
            ],

            Self::South2NorthEast => &[
                // Higher Elevation
                (Self::All, 1),
            ],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[
                //Higher Elevation
                (Self::West, 1),
            ],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[
                // Higher Elevation
                (Self::East, 1),
            ],

            Self::NorthWest2East => &[(Self::SouthWest, 0)],
            Self::SouthWest2NorthWest => &[(Self::West2SouthEast, 0)],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[(Self::SouthNorthWest, 0)],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],

            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::South2NorthEast, 0),
                (Self::WestSouthEast2, 0),
                (Self::All, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::SouthEastWest2NorthWest, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                (Self::SouthEastWest2NorthWest, 1),
                (Self::SouthWestEast2NorthEast, 1),
                (Self::SouthNorthWest, 1),
                (Self::East, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::North, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
                (Self::NorthWestEast2SouthEast, -1),
                (Self::NorthEastWest2SouthWest, -1),
            ],
        }
    }

    pub const fn valid_south_west_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::NorthEastWest2SouthWest, 0),
                (Self::WestSouthEast2, 0),
                (Self::All, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::East, -1),
                (Self::North, -1),
                (Self::NorthEast, -1),
                (Self::SouthNorthEast, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
                (Self::NorthWestEast2SouthEast, -1),
                (Self::SouthWestEast2NorthEast, -1),
                // Lower Elevation -2
                (Self::NorthWestEast2SouthEast, -2),
                // Higher Elevation
                (Self::None, 1),
                (Self::West, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                (Self::SouthNorthWest, 1),
                (Self::NorthWest, 1),
                (Self::NorthEast, 1),
                (Self::East, 1),
                (Self::All, 1),
                (Self::SouthWest2NorthWest, 1),
                // Higher Elevation 2x
                (Self::None, 2),
                (Self::SouthWestEast2NorthEast, 2),
                (Self::SouthEastWest2NorthWest, 2),
            ],

            // Terrain is NOT raised in the South-West
            Self::None
            | Self::North
            | Self::NorthWest
            | Self::NorthEast
            | Self::SouthEast
            | Self::NorthSouthEast
            | Self::East
            | Self::NorthWestEast2SouthEast => &[
                (Self::None, 0),
                (Self::West, 0),
                (Self::South, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::NorthWest, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::WestSouthEast2, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::East, -1),
                (Self::North, -1),
                (Self::NorthEast, -1),
                (Self::NorthWest2East, -1),
                (Self::SouthNorthEast, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
                (Self::NorthEastWest2SouthWest, -1),
                (Self::SouthWestEast2NorthEast, -1),
                // Lower Elevation -2
                (Self::NorthWestEast2SouthEast, -2),
            ],

            // Terrain is raised in the South-West
            Self::All
            | Self::South
            | Self::SouthWest
            | Self::West
            | Self::NorthSouthWest
            | Self::SouthNorthWest
            | Self::SouthNorthEast
            | Self::SouthWestEast2NorthEast
            | Self::NorthEastWest2SouthWest => &[
                (Self::All, 0),
                (Self::North, 0),
                (Self::NorthEast, 0),
                (Self::East, 0),
                (Self::SouthNorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::NorthEastWest2SouthWest, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::South2NorthEast, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::West, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                (Self::SouthNorthWest, 1),
                (Self::NorthWest, 1),
                (Self::NorthWestEast2SouthEast, 1),
                (Self::SouthEastWest2NorthWest, 1),
                (Self::West2SouthEast, 1),
                (Self::SouthWest2NorthWest, 1),
                // Lower Elevation
                (Self::NorthWestEast2SouthEast, -1),
                (Self::WestNorthEast2, -1),
            ],

            // Terrain is raised in the South-West 2x
            Self::SouthEastWest2NorthWest => &[
                (Self::None, 2),
                (Self::All, 1),
                (Self::North, 1),
                (Self::NorthEast, 1),
                (Self::East, 1),
                (Self::SouthNorthEast, 1),
                (Self::SouthWestEast2NorthEast, 1),
                (Self::NorthSouthEast, 1),
                (Self::NorthSouthWest, 1),
                // Higher Elevation 2x
                (Self::SouthEastWest2NorthWest, 2),
                (Self::SouthNorthWest, 2),
                (Self::West, 2),
                (Self::South, 2),
                (Self::SouthWest, 2),
            ],

            Self::South2NorthEast => &[
                // Higher Elevation
                (Self::East, 1),
            ],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[
                // Higher Elevation x2
                (Self::None, 2),
            ],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[
                // Higher Elevation
                (Self::None, 1),
            ],

            Self::NorthWest2East => &[(Self::SouthNorthWest, 0)],
            Self::SouthWest2NorthWest => &[],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[
                // Higher Elevation
                (Self::West, 1),
            ],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],
        }
    }

    pub const fn valid_south_east_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            TerrainSlope::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::South2NorthEast, 0),
                (Self::WestSouthEast2, 0),
                (Self::All, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::NorthWestEast2SouthEast, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::West, -1),
                (Self::North, -1),
                (Self::NorthWest, -1),
                (Self::SouthNorthWest, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
                (Self::NorthEastWest2SouthWest, -1),
                // Lower Elevation -2
                (Self::NorthEastWest2SouthWest, -2),
                // Higher Elevation
                (Self::None, 1),
                (Self::East, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                (Self::SouthNorthEast, 1),
                (Self::NorthEast, 1),
            ],
            // Terrain is NOT raised in the South-East
            Self::None
            | Self::North
            | Self::West
            | Self::NorthWest
            | Self::NorthEast
            | Self::SouthWest
            | Self::NorthSouthWest
            | Self::NorthEastWest2SouthWest => &[
                (Self::None, 0),
                (Self::South, 0),
                (Self::SouthNorthEast, 0),
                (Self::NorthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::East, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::South2NorthEast, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::West, -1),
                (Self::North, -1),
                (Self::NorthWest, -1),
                (Self::SouthNorthWest, -1),
                (Self::NorthSouthWest, -1),
                (Self::NorthSouthEast, -1),
                (Self::NorthWestEast2SouthEast, -1),
                (Self::SouthEastWest2NorthWest, -1),
                (Self::WestNorthEast2, -1),
                // Lower Elevation -2
                (Self::NorthEastWest2SouthWest, -2),
            ],

            // Terrain is raised in the South-East
            Self::South
            | Self::East
            | Self::SouthEast
            | Self::SouthNorthEast
            | Self::SouthNorthWest
            | Self::NorthSouthEast
            | Self::NorthWestEast2SouthEast
            | Self::SouthEastWest2NorthWest
            | Self::All => &[
                (Self::All, 0),
                (Self::West, 0),
                (Self::NorthWest, 0),
                (Self::SouthNorthWest, 0),
                (Self::North, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::NorthWestEast2SouthEast, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::WestSouthEast2, 0),
                (Self::SouthWest2NorthWest, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::East, 1),
                (Self::South, 1),
                (Self::SouthWest, 1),
                (Self::SouthEast, 1),
                (Self::SouthNorthEast, 1),
                (Self::NorthEast, 1),
                (Self::SouthWestEast2NorthEast, 1),
                // Lower Elevation
                (Self::NorthEastWest2SouthWest, -1),
                (Self::NorthWest2East, -1),
                (Self::West2SouthEast, -1),
            ],

            // Terrain is raised in the South-East 2x
            Self::SouthWestEast2NorthEast => &[
                (Self::All, 1),
                (Self::NorthWest, 1),
                (Self::SouthNorthWest, 1),
                (Self::North, 1),
                (Self::West, 1),
                (Self::None, 2),
                (Self::SouthEast, 2),
                (Self::East, 2),
                (Self::South, 2),
            ],

            Self::South2NorthEast => &[
                // Higher Elevation 2x
                (Self::None, 2),
            ],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[(Self::SouthNorthWest, 0)],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[
                // Higher Elevation
                (Self::All, 1),
            ],

            Self::NorthWest2East => &[(Self::East, 0)],
            Self::SouthWest2NorthWest => &[(Self::SouthWest, 0)],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[(Self::SouthWest, 0)],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],
        }
    }

    pub const fn valid_west_neighbors(&self) -> &'static [(Self, i8)] {
        match self {
            // West is not raised at all
            Self::None | Self::East | Self::SouthEast | Self::NorthEast => &[
                (Self::None, 0),
                (Self::West, 0),
                (Self::SouthWest, 0),
                (Self::NorthWest, 0),
                (Self::All, -1),
                (Self::East, -1),
                (Self::SouthNorthEast, -1),
                (Self::NorthSouthEast, -1),
            ],

            // West is raised
            Self::West | Self::SouthNorthWest | Self::NorthSouthWest | Self::All => &[
                (Self::All, 0),
                (Self::East, 0),
                (Self::SouthNorthEast, 0),
                (Self::NorthSouthEast, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::West, 1),
                (Self::NorthWest, 1),
                (Self::SouthWest, 1),
                (Self::SouthWest2NorthWest, 1),
            ],

            // West is raised in the North
            Self::North
            | Self::NorthWest
            | Self::NorthSouthEast
            | Self::NorthWestEast2SouthEast => &[
                (Self::NorthEast, 0),
                (Self::NorthWest2East, 0),
                (Self::North, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthEastWest2SouthWest, 0),
                (Self::NorthWestEast2SouthEast, -1),
            ],

            Self::NorthEastWest2SouthWest => &[
                (Self::NorthWest, 1),
                (Self::NorthSouthEast, 1),
                (Self::North, 1),
                (Self::NorthSouthWest, 1),
                (Self::NorthEastWest2SouthWest, 1),
            ],

            // West is raised in the South
            Self::South
            | Self::SouthWest
            | Self::SouthNorthEast
            | Self::SouthWestEast2NorthEast => &[
                (Self::SouthEast, 0),
                (Self::South, 0),
                (Self::SouthNorthWest, 0),
                (Self::SouthEastWest2NorthWest, 0),
                (Self::West2SouthEast, 0),
                // Lower Elevation
                (Self::SouthWestEast2NorthEast, -1),
                (Self::South2NorthEast, -1),
            ],

            // West is raised in the South 2x
            Self::SouthEastWest2NorthWest => &[
                (Self::SouthEast, 1),
                (Self::SouthNorthEast, 1),
                (Self::South, 1),
                (Self::SouthNorthWest, 1),
                (Self::SouthEastWest2NorthWest, 1),
                (Self::NorthEastWest2SouthWest, 1),
            ],

            Self::South2NorthEast => &[(Self::WestSouthEast2, 0)],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[
                // Higher Elevation
                (Self::None, 1),
            ],

            Self::NorthWest2East => &[(Self::WestNorthEast2, 0)],
            Self::SouthWest2NorthWest => &[],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[
                // Higher Elevation
                (Self::West, 1),
            ],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],

            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::WestSouthEast2, 0),
                (Self::All, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::West, 1),
                (Self::SouthWest, 1),
                (Self::NorthWest, 1),
                (Self::SouthNorthWest, 1),
                (Self::SouthEastWest2NorthWest, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::East, -1),
                (Self::NorthSouthEast, -1),
                (Self::SouthNorthEast, -1),
                (Self::NorthWestEast2SouthEast, -1),
                (Self::SouthWestEast2NorthEast, -1),
            ],
        }
    }

    pub const fn valid_east_neighbor(&self) -> &'static [(Self, i8)] {
        match self {
            // East is not raised at all
            Self::None | Self::West | Self::NorthWest | Self::SouthWest => &[
                (Self::None, 0),
                (Self::East, 0),
                (Self::NorthEast, 0),
                (Self::SouthEast, 0),
                // Lower Elevation
                (Self::All, -1),
                (Self::West, -1),
                (Self::SouthNorthWest, -1),
                (Self::NorthSouthWest, -1),
                (Self::WestSouthEast2, -1),
                (Self::WestNorthEast2, -1),
            ],

            // East is raised
            Self::East | Self::SouthNorthEast | Self::NorthSouthEast | Self::All => &[
                (Self::All, 0),
                (Self::West, 0),
                (Self::SouthNorthWest, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthWestEast2SouthEast, 0),
                // Lower Elevation
                (Self::West2SouthEast, -1),
                // Higher Elevation
                (Self::None, 1),
                (Self::East, 1),
                (Self::NorthEast, 1),
                (Self::SouthEast, 1),
            ],

            // East is raised in the North
            Self::North
            | Self::NorthEast
            | Self::NorthSouthWest
            | Self::NorthEastWest2SouthWest => &[
                (Self::NorthWest, 0),
                (Self::North, 0),
                (Self::NorthSouthEast, 0),
                (Self::NorthWestEast2SouthEast, 0),
                (Self::NorthEastWest2SouthWest, -1),
            ],

            // East is raised in the North 2x
            Self::NorthWestEast2SouthEast => &[
                (Self::North, 1),
                (Self::NorthWest, 1),
                (Self::NorthSouthEast, 1),
                (Self::NorthWestEast2SouthEast, 1),
            ],

            // East is raised in the South
            Self::South
            | Self::SouthEast
            | Self::SouthNorthWest
            | Self::SouthEastWest2NorthWest => &[
                (Self::SouthWest, 0),
                (Self::South, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::South2NorthEast, 0),
                // Lower Elevation
                (Self::SouthEastWest2NorthWest, -1),
                (Self::SouthWest2NorthWest, -1),
            ],

            // East is raised in the South 2x
            Self::SouthWestEast2NorthEast => &[
                (Self::SouthWest, 1),
                (Self::South, 1),
                (Self::SouthNorthEast, 1),
                (Self::SouthWestEast2NorthEast, 1),
            ],

            Self::South2NorthEast => &[
                // Higher Elevation
                (Self::South, 1),
            ],
            Self::East2NorthWest => &[],
            Self::North2SouthWest => &[],
            Self::West2SouthEast => &[(Self::SouthWest, 0)],

            Self::South2NorthWest => &[],
            Self::East2SouthWest => &[],
            Self::North2SouthEast => &[],
            Self::West2NorthEast => &[],

            Self::SouthNorthEast2 => &[],
            Self::EastNorthWest2 => &[],
            Self::NorthSouthWest2 => &[],
            Self::WestSouthEast2 => &[(Self::South2NorthEast, 0)],

            Self::NorthWest2East => &[(Self::NorthWestEast2SouthEast, 0)],
            Self::SouthWest2NorthWest => &[
                // Lower Elevation
                (Self::SouthNorthWest, -1),
            ],
            Self::SouthEast2West => &[],
            Self::NorthEast2SouthEast => &[],

            Self::WestNorthEast2 => &[(Self::NorthWest2East, 0)],
            Self::SouthNorthWest2 => &[],
            Self::EastSouthWest2 => &[],
            Self::NorthSouthEast2 => &[],

            // verticalL cliff for waterfall
            Self::VertialCliff => &[],

            Self::Undetermined => &[
                (Self::None, 0),
                (Self::North, 0),
                (Self::South, 0),
                (Self::West, 0),
                (Self::East, 0),
                (Self::NorthWest, 0),
                (Self::NorthEast, 0),
                (Self::NorthWest2East, 0),
                (Self::NorthSouthWest, 0),
                (Self::NorthSouthEast, 0),
                (Self::SouthWest, 0),
                (Self::SouthEast, 0),
                (Self::SouthNorthEast, 0),
                (Self::SouthNorthWest, 0),
                (Self::All, 0),
                (Self::SouthWestEast2NorthEast, 0),
                (Self::NorthWestEast2SouthEast, 0),
                // Higher Elevation
                (Self::None, 1),
                (Self::East, 1),
                (Self::SouthEast, 1),
                (Self::NorthEast, 1),
                (Self::NorthSouthEast, 1),
                (Self::North, 1),
                (Self::NorthWestEast2SouthEast, 1),
                (Self::SouthWestEast2NorthEast, 1),
                // Lower Elevation
                (Self::All, -1),
                (Self::West, -1),
                (Self::NorthSouthWest, -1),
                (Self::SouthNorthWest, -1),
                (Self::NorthEastWest2SouthWest, -1),
                (Self::SouthEastWest2NorthWest, -1),
            ],
        }
    }
}

impl TryFrom<u8> for TerrainSlope {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let slope = match value {
            0x00 => Self::None,
            0x01 => Self::North,
            0x02 => Self::East,
            0x03 => Self::South,
            0x04 => Self::West,
            0x05 => Self::NorthSouthEast,
            0x06 => Self::SouthNorthEast,
            0x07 => Self::SouthNorthWest,
            0x08 => Self::NorthSouthWest,
            0x09 => Self::NorthEast,
            0x0A => Self::SouthEast,
            0x0B => Self::SouthWest,
            0x0C => Self::NorthWest,
            0x0D => Self::All,
            0x0E => Self::VertialCliff,

            0x0F.. => {
                logger::warn!(
                    "TerrainSlope value is out of range: {}. Using 0 instead.",
                    value
                );
                Self::None
            }
        };

        Ok(slope)
    }
}

impl From<TerrainSlope> for u8 {
    fn from(value: TerrainSlope) -> Self {
        match value {
            TerrainSlope::None => 0x00,
            TerrainSlope::North => 0x01,
            TerrainSlope::East => 0x02,
            TerrainSlope::South => 0x03,
            TerrainSlope::West => 0x04,
            TerrainSlope::NorthSouthEast => 0x05,
            TerrainSlope::SouthNorthEast => 0x06,
            TerrainSlope::SouthNorthWest => 0x07,
            TerrainSlope::NorthSouthWest => 0x08,
            TerrainSlope::NorthEast => 0x09,
            TerrainSlope::SouthEast => 0x0A,
            TerrainSlope::SouthWest => 0x0B,
            TerrainSlope::NorthWest => 0x0C,
            TerrainSlope::All => 0x0D,
            TerrainSlope::VertialCliff => 0xE,
            TerrainSlope::Undetermined => {
                panic!("The undetermined type can not be converted to u8")
            }

            TerrainSlope::NorthWestEast2SouthEast
            | TerrainSlope::NorthEastWest2SouthWest
            | TerrainSlope::SouthWestEast2NorthEast
            | TerrainSlope::SouthEastWest2NorthWest
            | TerrainSlope::South2NorthEast
            | TerrainSlope::East2NorthWest
            | TerrainSlope::North2SouthWest
            | TerrainSlope::West2SouthEast
            | TerrainSlope::SouthNorthEast2
            | TerrainSlope::EastNorthWest2
            | TerrainSlope::NorthSouthWest2
            | TerrainSlope::WestSouthEast2
            | TerrainSlope::NorthWest2East
            | TerrainSlope::SouthWest2NorthWest
            | TerrainSlope::SouthEast2West
            | TerrainSlope::NorthEast2SouthEast
            | TerrainSlope::WestNorthEast2
            | TerrainSlope::SouthNorthWest2
            | TerrainSlope::EastSouthWest2
            | TerrainSlope::NorthSouthEast2
            | TerrainSlope::South2NorthWest
            | TerrainSlope::East2SouthWest
            | TerrainSlope::North2SouthEast
            | TerrainSlope::West2NorthEast => {
                panic!("The 2x types can not be converted to u8")
            }
        }
    }
}
