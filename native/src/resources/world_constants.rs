use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base = Resource)]
pub struct WorldConstants {
    #[export]
    #[var(get = tile_size, set = set_tile_size)]
    tile_size: u8,

    #[export]
    #[var(get = tile_height, set = set_tile_height)]
    tile_height: u8,
}

#[godot_api]
impl WorldConstants {
    #[func]
    pub fn tile_size(&self) -> u8 {
        self.tile_size
    }

    #[func]
    fn set_tile_size(&mut self, value: u8) {
        self.tile_size = value;
    }

    #[func]
    pub fn tile_height(&self) -> u8 {
        self.tile_height
    }

    #[func]
    fn set_tile_height(&mut self, value: u8) {
        self.tile_height = value;
    }
}
