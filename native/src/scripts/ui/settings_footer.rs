use godot::classes::Control;
use godot::obj::Gd;
use godot::{builtin::Vector2, classes::notify::CanvasItemNotification};
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsFooter {
    #[export(range(min = 0.0, max = 1.0, step = 0.001))]
    #[prop(set = Self::set_animation_offset)]
    pub animation_offset: f32,

    original_position: Option<Vector2>,
    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsFooter {
    pub fn _notification(&mut self, what: i32) {
        logger::debug!("footer notification: {}", what);

        if CanvasItemNotification::from(what) == CanvasItemNotification::TRANSFORM_CHANGED
            && self.base.get_position() != Vector2::ZERO
        {
            self.original_position = Some(self.base.get_position());
            self.base.set_notify_transform(false);
        }
    }

    pub fn _ready(&mut self) {
        self.base.set_notify_transform(true);
    }

    pub fn _process(&mut self, _delta: f32) {
        let original_position = self.base.get_position();

        if original_position != Vector2::ZERO && self.original_position.is_none() {
            self.original_position = Some(original_position);
        }
    }

    fn set_animation_offset(&mut self, value: f32) {
        self.animation_offset = value;

        let Some(original_position) = self.original_position else {
            return;
        };

        let offset = self.base.get_size().y * value;

        self.base
            .set_position(original_position + Vector2::new(0.0, offset));
    }
}
