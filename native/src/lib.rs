#[cfg(debug_assertions)]
mod editor;
mod ext;
mod objects;
mod project_settings;
mod resources;
mod road_navigation;
mod scripts;
mod terrain_builder;
mod util;
mod world;

use godot::{
    init::InitStage,
    prelude::{gdextension, ExtensionLibrary},
};

struct NativeLib;

#[gdextension]
unsafe impl ExtensionLibrary for NativeLib {
    fn on_stage_init(level: InitStage) {
        if level == InitStage::Scene {
            godot_rust_script::init!(scripts);
        }
    }

    fn on_stage_deinit(level: InitStage) {
        if level == InitStage::Scene {
            godot_rust_script::deinit!();
        }
    }
}

#[macro_export]
macro_rules! class_callable {
    ($instance:expr, $host:ident::$fn:ident) => {{
        let instance: &$host = &*$instance;

        let _fn_ptr = $host::$fn;

        instance.base().callable(stringify!($fn))
    }};
}

#[macro_export]
macro_rules! script_callable {
    ($instance:expr, $host:ident::$fn:ident) => {{
        let instance: &$host = &*$instance;

        let _fn_ptr = $host::$fn;

        instance.base.callable(stringify!($fn))
    }};
}

#[macro_export]
macro_rules! engine_callable {
    ($instance:expr, $host:ident::$fn:ident) => {{
        fn __typecheck<T: ::godot::obj::Inherits<$host>>(instance: &Gd<T>) -> &Gd<T> {
            instance
        }

        let _fn_ptr = $host::$fn;

        __typecheck($instance).callable(stringify!($fn))
    }};
}
