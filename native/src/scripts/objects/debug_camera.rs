use godot::builtin::Vector3;
use godot::classes::{Camera3D, InputEvent};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor};

use crate::resources::InputDevice;

#[derive(GodotScript, Debug)]
struct DebugCamera {
    #[export]
    pub input_device: OnEditor<Gd<InputDevice>>,

    #[export]
    pub move_speed: f32,

    base: Gd<Camera3D>,
}

#[godot_script_impl]
impl DebugCamera {
    pub fn _physics_process(&mut self, delta: f32) {
        let input_device = self.input_device.bind();

        let x = input_device.strafe_strength() * self.move_speed * delta;
        let y = input_device.movement_strength() * self.move_speed * delta;

        self.base.translate(Vector3 { x, y: 0.0, z: y });
    }

    pub fn _unhandled_input(&mut self, event: Gd<InputEvent>) {
        self.input_device.bind_mut().capture(event);
    }
}
