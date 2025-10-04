use godot::builtin::Vector3;
use godot::classes::Resource;
use godot::global::godot_error;
use godot::obj::Gd;

#[derive(Debug, Default)]
pub struct CityCoordsFeature {
    world_constants: Gd<Resource>,
    sea_level: u32,
}

impl CityCoordsFeature {
    pub fn new(world_constants: Gd<Resource>, sea_level: u32) -> Self {
        Self {
            world_constants,
            sea_level,
        }
    }

    pub fn get_world_coords(&self, x: u32, y: u32, z: u32) -> Vector3 {
        let tile_size: u8 = self.world_constants.get("tile_size").to();
        let tile_height: u32 = self.world_constants.get("tile_height").to();

        Vector3 {
            x: (x * u32::from(tile_size)) as f32,
            y: (z.max(self.sea_level - 1) * tile_height) as f32,
            z: (y * u32::from(tile_size)) as f32,
        }
    }

    pub fn get_building_coords(&self, x: u32, y: u32, z: u32, size: u8) -> Vector3 {
        let tile_size: u8 = self
            .world_constants
            .get("tile_size")
            .try_to()
            .map_err(|err| godot_error!("failed to get tile_size of world_constants: {}", err))
            .unwrap();

        let offset = (size * tile_size) as f32 / 2.0;

        // OpenCity2k gets the bottom left corner, we have to correct that.
        let y = y - (size as u32 - 1);

        let mut location = self.get_world_coords(x, y, z);

        location.x += offset;
        location.z += offset;

        location
    }
}
