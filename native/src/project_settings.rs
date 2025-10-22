use godot::classes;

pub trait CustomProjectSettings {
    const DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_NETWORK: &str =
        "debug/shapes/road_navigation/display_network";
    const DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET: &str =
        "debug/shapes/road_navigation/display_vehicle_target";
    const EDITOR_REQUIRED_VERSION: &str = "editor/required_version";
}

impl CustomProjectSettings for classes::ProjectSettings {}
