use godot::builtin::Vector2;
use godot::classes::Control;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript};

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsTabBar {
    #[prop(set = Self::set_animation_offset)]
    #[export(range(min = 0.0, max = 1.0, step = 0.001))]
    pub animation_offset: f32,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsTabBar {
    fn set_animation_offset(&mut self, value: f32) {
        self.animation_offset = value;

        let offset = self.base.get_size().y * -value;

        self.base.set_position(Vector2::new(0.0, offset));
    }
}
