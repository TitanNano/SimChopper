use godot::builtin::{Dictionary, GString};
use godot::classes::{Node3D, RichTextLabel};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor};
use itertools::Itertools;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct Debugger3D {
    #[export]
    pub title: GString,

    #[export]
    pub text_view: OnEditor<Gd<RichTextLabel>>,

    pub debug_data: Dictionary,
}

#[godot_script_impl]
impl Debugger3D {
    pub fn _process(&mut self, _delta: f64) {
        let title = &self.title;

        let values = self
            .debug_data
            .iter_shared()
            .map(|(key, val)| format!("{}: {}", key, val))
            .join("\n");

        self.text_view.set_text(&format!("{}\n\n{}", title, values));
    }
}
