/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use godot::classes::{GpuParticles3D, Node3D};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, GodotScriptEnum, OnEditor};

#[derive(Debug, Default, GodotScriptEnum, Clone, Copy)]
#[script_enum(export)]
pub enum CanonMode {
    #[default]
    Inactive,
    Water,
    Teargas,
}

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct CanonUpgrade {
    #[export]
    #[prop(set = Self::set_mode)]
    pub mode: CanonMode,

    #[export]
    pub water_jet: OnEditor<Gd<GpuParticles3D>>,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl CanonUpgrade {
    pub fn _ready(&mut self) {
        self.set_mode(self.mode);
    }

    pub fn set_mode(&mut self, value: CanonMode) {
        self.mode = value;

        if !self.base.is_node_ready() {
            return;
        }

        self.water_jet.set_emitting(false);

        match value {
            CanonMode::Water => self.water_jet.set_emitting(true),
            CanonMode::Teargas | CanonMode::Inactive => (),
        }
    }

    pub fn action(&mut self, pressed: bool) {
        match (pressed, self.mode) {
            (true, CanonMode::Inactive) => {
                self.set_mode(CanonMode::Water);
            }

            (false, CanonMode::Water) => {
                self.set_mode(CanonMode::Inactive);
            }

            _ => (),
        }
    }
}
