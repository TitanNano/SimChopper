use godot::builtin::GString;
use godot::classes::{Control, Label, Texture2D, TextureRect};
use godot::obj::Gd;
use godot::prelude::Var;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptExportGroup};

use crate::resources::InputPromptTexture;

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<TextureRect>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsOptionInput {
    #[export(flatten)]
    pub nodes: ChildNodes,

    #[export]
    #[prop(set = Self::set_label)]
    pub label: GString,

    #[export]
    #[prop(set = Self::set_texture)]
    pub texture: OnEditor<Gd<InputPromptTexture>>,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsOptionInput {
    pub fn _ready(&mut self) {
        self.set_texture(self.texture.clone());
        self.set_label(self.label.clone());
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    fn set_texture(&mut self, texture: Gd<InputPromptTexture>) {
        if self.base.is_node_ready() {
            self.nodes
                .value
                .set_texture(&texture.clone().upcast::<Texture2D>());
        }

        Var::var_set(&mut self.texture, Some(texture));
    }
}
