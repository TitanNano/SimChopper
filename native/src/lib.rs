mod terrain_builder;

use std::cell::RefCell;

use godot::prelude::{gdextension, ExtensionLibrary, InitLevel};
use godot_rust_script::{self, RustScriptExtensionLayer};

godot_rust_script::setup!(scripts);

struct NativeLib;

thread_local! {
    static RUST_SCRIPT_LAYER: RefCell<RustScriptExtensionLayer> = RefCell::new(godot_rust_script::init!());
}

#[gdextension]
unsafe impl ExtensionLibrary for NativeLib {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Core => (),
            InitLevel::Servers => (),
            InitLevel::Scene => {
                RUST_SCRIPT_LAYER.with_borrow_mut(|layer| layer.initialize());
            }
            InitLevel::Editor => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Editor => (),
            InitLevel::Scene => RUST_SCRIPT_LAYER.with_borrow_mut(|layer| layer.deinitialize()),
            InitLevel::Servers => {}
            InitLevel::Core => (),
        }
    }
}
