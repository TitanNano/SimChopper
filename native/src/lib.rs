mod terrain_builder;

use godot::prelude::{gdextension, ExtensionLibrary};

struct NativeLib;

#[gdextension]
unsafe impl ExtensionLibrary for NativeLib {}
