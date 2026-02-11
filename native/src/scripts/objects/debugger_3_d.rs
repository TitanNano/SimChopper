use godot::builtin::{GString, VarDictionary};
use godot::classes::{Node3D, RichTextLabel};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor};
use itertools::Itertools;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
pub struct Debugger3D {
    #[export]
    pub title: GString,

    #[export]
    pub text_view: OnEditor<Gd<RichTextLabel>>,

    debug_data: VarDictionary,
}

#[godot_script_impl]
impl Debugger3D {
    pub fn _process(&mut self, _delta: f64) {
        let title = &self.title;

        let values = self
            .debug_data
            .iter_shared()
            .map(|(key, val)| format!("{key}: {val}"))
            .join("\n");

        self.text_view.set_text(&format!("{title}\n\n{values}"));
    }

    pub fn debug_data(&self) -> VarDictionary {
        self.debug_data.clone()
    }
}
