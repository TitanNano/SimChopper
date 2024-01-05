mod scripts;
mod terrain_builder;

use godot::prelude::{gdextension, ExtensionLibrary, InitLevel};
use godot_rust_script;

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
