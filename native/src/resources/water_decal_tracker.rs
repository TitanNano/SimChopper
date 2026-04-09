/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::collections::HashSet;

use godot::builtin::Aabb;
use godot::classes::{Decal, IResource, Resource};
use godot::obj::{Base, Gd};
use godot::prelude::godot_api;
use godot::register::GodotClass;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct WaterDecalTracker {
    decals: HashSet<Gd<Decal>>,
    base: Base<Resource>,
}

impl WaterDecalTracker {
    pub(crate) fn insert(&mut self, decal: &Gd<Decal>) {
        self.decals.insert(decal.clone());
    }

    pub(crate) fn get_decals_at_point(&self, point: Aabb) -> Box<[&Gd<Decal>]> {
        self.decals
            .iter()
            .filter(|decal| point.contains_point(decal.get_global_position()))
            .collect()
    }

    pub(crate) fn free(&mut self, decal: &Gd<Decal>) {
        self.decals.remove(decal);
    }
}

#[godot_api]
impl IResource for WaterDecalTracker {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            decals: HashSet::default(),
            base,
        }
    }
}
