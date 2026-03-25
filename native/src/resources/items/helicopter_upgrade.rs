/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use godot::builtin::{GString, StringName};
use godot::classes::{PackedScene, Resource};
use godot::obj::{Base, Gd};
use godot::prelude::GodotClass;

#[derive(GodotClass)]
#[class(base = Resource, init)]
struct HelicopterUpgrade {
    /// Name of the upgrade
    #[export]
    name: GString,

    /// The game object scene for this upgrade. It will be attached to the helicopter.
    #[export]
    object: Option<Gd<PackedScene>>,

    /// The input action which will enable or disable the game object of this upgrade.
    #[export]
    action: StringName,

    /// The price of this upgrade.
    #[export]
    price: u32,

    base: Base<Resource>,
}