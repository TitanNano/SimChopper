#[cfg(debug_assertions)]
mod editor;
mod objects;
mod resources;
mod road_navigation;
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

#[macro_export]
macro_rules! callable {
    ($path:ty, $fn:ident, $instance:expr) => {{
        let inst: &Gd<$path> = $instance;
        let _fn_ptr = <$path>::$fn;

        godot::builtin::Callable::from_object_method(inst, stringify!($fn))
    }};
}
