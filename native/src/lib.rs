mod panic;
mod terrain_builder;

use gdnative::prelude::*;
use panic::init_panic_hook;
use terrain_builder::{TerrainBuilder, TerrainBuilderFactory, TerrainRotation};

fn init(handle: InitHandle) {
    handle.add_class::<TerrainBuilder>();
    handle.add_class::<TerrainBuilderFactory>();
    handle.add_class::<TerrainRotation>();

    init_panic_hook();
}

godot_init!(init);
