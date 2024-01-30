mod objects;
mod scripts;
mod terrain_builder;
mod util;
mod world;

use godot::prelude::{gdextension, ExtensionLibrary, InitLevel};

struct NativeLib;

#[gdextension]
unsafe impl ExtensionLibrary for NativeLib {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Core => (),
            InitLevel::Servers => (),
            InitLevel::Scene => godot_rust_script::init!(scripts),
            InitLevel::Editor => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Editor => (),
            InitLevel::Scene => godot_rust_script::deinit!(),
            InitLevel::Servers => (),
            InitLevel::Core => (),
        }
    }
}
