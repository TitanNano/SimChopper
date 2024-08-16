use godot::{
	builtin::{Dictionary, GString, NodePath},
	engine::{Node3D, RichTextLabel},
	obj::Gd,
};
use godot_rust_script::{godot_script_impl, GodotScript};
use itertools::Itertools;

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct Debugger3D {
	#[export]
	pub title: GString,

	#[export(node_path = ["RichTextLabel"])]
	pub text_view_path: NodePath,

	pub debug_data: Dictionary,

	text_view: Option<Gd<RichTextLabel>>,

	base: Gd<Node3D>,
}

#[godot_script_impl]
impl Debugger3D {
	pub fn _ready(&mut self) {
		self.text_view = self.base.try_get_node_as(self.text_view_path.clone());
	}

	pub fn _process(&mut self, _delta: f64) {
		let title = &self.title;

		let values = self
			.debug_data
			.iter_shared()
			.map(|(key, val)| format!("{}: {}", key, val))
			.join("\n");

		let Some(text_view) = self.text_view.as_mut() else {
			logger::error!("Debugger text view is unavailable!");
			return;
		};

		text_view.set_text(format!("{}\n\n{}", title, values).into());
	}
}
