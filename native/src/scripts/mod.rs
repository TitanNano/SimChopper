/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

mod effects;
mod objects;
mod particles;
mod spawner;
mod ui;
mod world;

pub use spawner::*;

godot_rust_script::define_script_root!();