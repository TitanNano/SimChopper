use godot::engine::{SceneTree, SceneTreeTimer};
use godot::obj::Gd;

pub mod async_support;
pub mod logger;

/// Create a new ingame one-shot timer in seconds.
pub fn timer(tree: &mut Gd<SceneTree>, delay: f64) -> Gd<SceneTreeTimer> {
    tree.create_timer_ex(delay)
        .process_always(false)
        .ignore_time_scale(false)
        .process_in_physics(true)
        .done()
        .unwrap()
}
