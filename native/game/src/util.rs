/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

pub mod async_support;
pub mod logger;
mod numbers;

use godot::builtin::{Basis, Vector3};
use godot::classes::{SceneTree, SceneTreeTimer};
use godot::obj::Gd;

pub use numbers::*;

/// Create a new in game one-shot timer in seconds.
#[inline]
pub fn timer(tree: &mut Gd<SceneTree>, delay: f64) -> Gd<SceneTreeTimer> {
    tree.create_timer_ex(delay)
        .process_always(false)
        .ignore_time_scale(false)
        .process_in_physics(true)
        .done()
        .unwrap()
}

pub(crate) mod vector3 {
    use godot::builtin::Vector3;

    #[expect(unused)]
    pub const XY_PLANE: Vector3 = Vector3 {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    };

    pub const XZ_PLANE: Vector3 = Vector3 {
        x: 1.0,
        y: 0.0,
        z: 1.0,
    };

    #[expect(unused)]
    pub const YZ_PLANE: Vector3 = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 1.0,
    };
}

#[inline]
pub(crate) fn basis_from_normal(normal: Vector3) -> Basis {
    Basis::from_cols(
        normal.cross(Basis::IDENTITY.col_c()),
        normal,
        Basis::IDENTITY.col_a().cross(normal),
    )
}

#[macro_export]
macro_rules! debug_3d {
    ($debugger: expr => $($variable: tt),+) => {
        #[cfg(debug_assertions)]
        if let Some(ref mut debugger) = $debugger {
            use $crate::scripts::objects::debugger_3_d::IDebugger3D;

            $(
                $crate::debug_3d!(inner debugger, $variable);
            )+
        }
    };

    (inner $debugger: ident, (float $variable: ident)) => {
        $debugger.debug_data().set(stringify!($variable), ($variable * 100.0).round() / 100.0);
    };

    (inner $debugger: ident, (as_deg $variable: ident)) => {
        $debugger.debug_data().set(stringify!($variable), $variable.to_degrees());
    };

    (inner $debugger: ident, (ref $variable: ident)) => {
        $debugger.debug_data().set(stringify!($variable), &$variable);
    };

    (inner $debugger: ident, $variable: ident) => {
        $debugger.debug_data().set(stringify!($variable), $variable);
    };
}
