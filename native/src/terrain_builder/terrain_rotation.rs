use godot::prelude::*;

use crate::world::city_data::TerrainSlope;

const TERAIN_ROTATION_CORNERS: [u8; 4] = [0, 1, 3, 2];

#[derive(GodotClass, Clone, Copy)]
#[class(base=RefCounted, init)]
pub(crate) struct TerrainRotation {
    offset: u8,
}

/// Terrain rotation is the number of counter clockwise rotations.
#[godot_api]
impl TerrainRotation {
    #[func]
    fn set_rotation(&mut self, rotation: i64) {
        self.offset = u8::try_from(rotation).unwrap_or(u8::MAX);
    }
}

impl TerrainRotation {
    fn get_corner(&self, index: u8) -> u8 {
        let shifted_index = ((index + self.offset) % 4) as usize;
        let target_value = TERAIN_ROTATION_CORNERS.get(shifted_index).unwrap_or(&0);

        target_value.to_owned()
    }

    pub fn nw(&self) -> usize {
        self.get_corner(0).into()
    }

    pub fn ne(&self) -> usize {
        self.get_corner(1).into()
    }

    pub fn se(&self) -> usize {
        self.get_corner(2).into()
    }

    pub fn sw(&self) -> usize {
        self.get_corner(3).into()
    }

    /// Apply one counter-clockwise roation to slope type.
    pub fn normalize_slope(&self, slope: TerrainSlope) -> TerrainSlope {
        let offset = self.offset % 4;
        let mut rotated_slope = slope;

        for _ in 0..offset {
            rotated_slope = match rotated_slope {
                TerrainSlope::None => TerrainSlope::None,
                TerrainSlope::All => TerrainSlope::All,

                TerrainSlope::North => TerrainSlope::West,
                TerrainSlope::South => TerrainSlope::East,
                TerrainSlope::West => TerrainSlope::South,
                TerrainSlope::East => TerrainSlope::North,

                TerrainSlope::NorthWest => TerrainSlope::SouthWest,
                TerrainSlope::NorthEast => TerrainSlope::NorthWest,
                TerrainSlope::SouthWest => TerrainSlope::SouthEast,
                TerrainSlope::SouthEast => TerrainSlope::NorthEast,

                TerrainSlope::NorthSouthWest => TerrainSlope::SouthNorthWest,
                TerrainSlope::NorthSouthEast => TerrainSlope::NorthSouthWest,
                TerrainSlope::SouthNorthWest => TerrainSlope::SouthNorthEast,
                TerrainSlope::SouthNorthEast => TerrainSlope::NorthSouthEast,

                TerrainSlope::VertialCliff => TerrainSlope::VertialCliff,

                TerrainSlope::NorthWestEast2SouthEast => TerrainSlope::NorthEastWest2SouthWest,
                TerrainSlope::NorthEastWest2SouthWest => TerrainSlope::SouthEastWest2NorthWest,
                TerrainSlope::SouthEastWest2NorthWest => TerrainSlope::SouthWestEast2NorthEast,
                TerrainSlope::SouthWestEast2NorthEast => TerrainSlope::NorthWestEast2SouthEast,

                TerrainSlope::South2NorthEast => TerrainSlope::East2NorthWest,
                TerrainSlope::East2NorthWest => TerrainSlope::North2SouthWest,
                TerrainSlope::North2SouthWest => TerrainSlope::West2SouthEast,
                TerrainSlope::West2SouthEast => TerrainSlope::South2NorthEast,

                TerrainSlope::South2NorthWest => TerrainSlope::East2SouthWest,
                TerrainSlope::East2SouthWest => TerrainSlope::North2SouthEast,
                TerrainSlope::North2SouthEast => TerrainSlope::West2NorthEast,
                TerrainSlope::West2NorthEast => TerrainSlope::South2NorthWest,

                TerrainSlope::SouthNorthEast2 => TerrainSlope::EastNorthWest2,
                TerrainSlope::EastNorthWest2 => TerrainSlope::NorthSouthWest2,
                TerrainSlope::NorthSouthWest2 => TerrainSlope::WestSouthEast2,
                TerrainSlope::WestSouthEast2 => TerrainSlope::SouthNorthEast2,

                TerrainSlope::NorthWest2East => TerrainSlope::SouthWest2NorthWest,
                TerrainSlope::SouthWest2NorthWest => TerrainSlope::SouthEast2West,
                TerrainSlope::SouthEast2West => TerrainSlope::NorthEast2SouthEast,
                TerrainSlope::NorthEast2SouthEast => TerrainSlope::NorthWest2East,

                TerrainSlope::WestNorthEast2 => TerrainSlope::SouthNorthWest2,
                TerrainSlope::SouthNorthWest2 => TerrainSlope::EastSouthWest2,
                TerrainSlope::EastSouthWest2 => TerrainSlope::NorthSouthEast2,
                TerrainSlope::NorthSouthEast2 => TerrainSlope::WestNorthEast2,

                TerrainSlope::Undetermined => TerrainSlope::Undetermined,
            };
        }

        rotated_slope
    }

    pub fn to_reverted(self) -> Self {
        let offset = 4 - self.offset;

        Self { offset }
    }
}
