use godot::prelude::*;

const TERAIN_ROTATION_CORNERS: [u8; 4] = [0, 1, 3, 2];

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct TerrainRotation {
    offset: u8,
}

#[godot_api]
impl TerrainRotation {
    #[func]
    fn set_rotation(&mut self, rotation: i64) {
        self.offset = u8::try_from(rotation).unwrap_or(u8::MAX);
    }
}

pub trait TerrainRotationBehaviour {
    fn get_corner(&self, index: u8) -> u8;

    fn nw(&self) -> usize;
    fn ne(&self) -> usize;
    fn se(&self) -> usize;
    fn sw(&self) -> usize;
}

impl TerrainRotationBehaviour for TerrainRotation {
    fn get_corner(&self, index: u8) -> u8 {
        let shifted_index = ((index + self.offset) % 4) as usize;
        let target_value = TERAIN_ROTATION_CORNERS.get(shifted_index).unwrap_or(&0);

        target_value.to_owned()
    }

    fn nw(&self) -> usize {
        self.get_corner(0).into()
    }

    fn ne(&self) -> usize {
        self.get_corner(1).into()
    }

    fn se(&self) -> usize {
        self.get_corner(2).into()
    }

    fn sw(&self) -> usize {
        self.get_corner(3).into()
    }
}
