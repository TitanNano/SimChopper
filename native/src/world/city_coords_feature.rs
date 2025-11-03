use godot::builtin::Vector3;
use godot::obj::Gd;
use num::ToPrimitive;

use crate::resources::WorldConstants;
use crate::util::Uf32;

#[derive(Debug, Default)]
pub struct CityCoordsFeature {
    world_constants: Gd<WorldConstants>,
    sea_level: Uf32,
}

impl CityCoordsFeature {
    pub fn new(world_constants: Gd<WorldConstants>, sea_level: u32) -> Self {
        Self {
            world_constants,
            sea_level: Uf32::new(sea_level),
        }
    }

    /// Transform tile coordinates to world translation.
    pub fn get_world_coords(&self, x: u32, y: u32, z: u32) -> Vector3 {
        let tile_size: Uf32 = Uf32::new(self.world_constants.bind().tile_size().into());
        let tile_height: Uf32 = Uf32::new(self.world_constants.bind().tile_height().into());
        let x = Uf32::new(x);
        let y = Uf32::new(y);
        let z = Uf32::new(z);

        Vector3 {
            x: (x * tile_size).into_f32(),
            y: (z.max(self.sea_level - Uf32::new(1)) * tile_height).into_f32(),
            z: (y * tile_size).into_f32(),
        }
    }

    /// Transform building coordinates to correct world translation.
    pub fn get_building_coords(&self, x: u32, y: u32, z: u32, size: u8) -> Vector3 {
        let tile_size = self.world_constants.bind().tile_size();

        let offset = f32::from(size * tile_size) / 2.0;

        // OpenCity2k gets the bottom left corner, we have to correct that.
        let y = y - (u32::from(size) - 1);

        let mut location = self.get_world_coords(x, y, z);

        location.x += offset;
        location.z += offset;

        location
    }

    /// Transforms world translation to tile coordinates with altitude.
    pub fn tile_coordinates(&self, translation: Vector3) -> (u32, u32, u32) {
        let tile_size = self.world_constants.bind().tile_size();
        let tile_height = self.world_constants.bind().tile_height();

        let x = (translation.x / f32::from(tile_size))
            .floor()
            .to_u32()
            .expect("x component should be positive");

        let y = (translation.z / f32::from(tile_size))
            .floor()
            .to_u32()
            .expect("y component should be positive");

        let altitude = (translation.y / f32::from(tile_height))
            .floor()
            .to_u32()
            .expect("altitude component should be positive");

        (x, y, altitude)
    }
}
