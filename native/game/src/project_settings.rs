/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use godot::classes;

pub trait CustomProjectSettings {
    #[expect(dead_code)]
    const DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_NETWORK: &str =
        "debug/shapes/road_navigation/display_network";
    const DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET: &str =
        "debug/shapes/road_navigation/display_vehicle_target";
    #[expect(dead_code)]
    const EDITOR_REQUIRED_VERSION: &str = "editor/required_version";
}

impl CustomProjectSettings for classes::ProjectSettings {}
