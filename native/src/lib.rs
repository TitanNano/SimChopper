/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

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

use godot::classes::{Engine, RefCounted, ResourceLoader, ResourceSaver};
use godot::init::InitStage;
use godot::obj::{Gd, NewGd, Singleton};
use godot::prelude::{gdextension, ExtensionLibrary, GodotClass};

use crate::resources::{TomlResourceLoader, TomlResourceSaver};
use crate::util::logger;

struct NativeLib;

#[gdextension]
unsafe impl ExtensionLibrary for NativeLib {
    fn on_stage_init(level: InitStage) {
        if level == InitStage::Scene {
            godot_rust_script::init!(scripts);

            let toml_resource_loader = TomlResourceLoader::new_gd();

            ResourceLoader::singleton().add_resource_format_loader(&toml_resource_loader);

            Engine::singleton().register_singleton(
                &TomlResourceLoader::class_id().to_string_name(),
                &RcSingletonWrapper::wrap(toml_resource_loader.upcast()),
            );

            let toml_resource_saver = TomlResourceSaver::new_gd();

            ResourceSaver::singleton().add_resource_format_saver(&toml_resource_saver);

            Engine::singleton().register_singleton(
                &TomlResourceSaver::class_id().to_string_name(),
                &RcSingletonWrapper::wrap(toml_resource_saver.upcast()),
            );
        }
    }

    fn on_stage_deinit(level: InitStage) {
        if level == InitStage::Scene {
            godot_rust_script::deinit!();

            let mut engine = Engine::singleton();
            let mut resource_loader = ResourceLoader::singleton();
            let mut resource_saver = ResourceSaver::singleton();

            // De-register TOML resource loader.
            if let Some(inst) =
                engine.get_singleton(&TomlResourceLoader::class_id().to_string_name())
            {
                let wrapped_toml_resource_loader = inst.cast::<RcSingletonWrapper>();
                let toml_resource_loader = wrapped_toml_resource_loader
                    .bind()
                    .inner
                    .clone()
                    .cast::<TomlResourceLoader>();

                resource_loader.remove_resource_format_loader(&toml_resource_loader);

                engine.unregister_singleton(&TomlResourceLoader::class_id().to_string_name());
                wrapped_toml_resource_loader.free();
            } else {
                logger::error!(
                    "Unable to access unknown {} singleton!",
                    TomlResourceLoader::class_id()
                );
            }

            // De-register TOML resource saver.
            if let Some(inst) =
                engine.get_singleton(&TomlResourceSaver::class_id().to_string_name())
            {
                let wrapped_toml_resource_saver = inst.cast::<RcSingletonWrapper>();
                let toml_resource_saver = wrapped_toml_resource_saver
                    .bind()
                    .inner
                    .clone()
                    .cast::<TomlResourceSaver>();

                resource_saver.remove_resource_format_saver(&toml_resource_saver);

                engine.unregister_singleton(&TomlResourceSaver::class_id().to_string_name());
                wrapped_toml_resource_saver.free();
            } else {
                logger::error!(
                    "Unable to access unknown {} singleton!",
                    TomlResourceSaver::class_id()
                );
            }
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

#[derive(GodotClass)]
#[class(base = Object, init)]
struct RcSingletonWrapper {
    inner: Gd<RefCounted>,
}

impl RcSingletonWrapper {
    fn wrap(inner: Gd<RefCounted>) -> Gd<Self> {
        let slf = Self { inner };

        Gd::from_object(slf)
    }
}
