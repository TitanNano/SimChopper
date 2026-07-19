use godot::builtin::GString;
use godot::classes::{Control, Label, Texture2D, TextureRect};
use godot::obj::Gd;
use godot::prelude::Var;
use godot_rust_script::{
    godot_script_impl, GodotScript, OnEditor, Rs, RsDynify, ScriptExportGroup,
};

use crate::resources::InputPromptTexture;
use crate::scripts::ui::tab_controller::SettingsControl;

#[derive(ScriptExportGroup, Debug, Default)]
pub(crate) struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<TextureRect>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
pub(crate) struct SettingsOptionInput {
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
        self.set_label(self.label.clone());

        if Var::var_get(&self.texture).is_some() {
            self.set_texture(self.texture.clone());
        }
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

impl SettingsControl for Rs<SettingsOptionInput> {
    fn grab_focus(&mut self) {}
}

impl RsDynify<dyn SettingsControl> for SettingsOptionInput {
    fn coerce(source: Rs<Self>) -> Box<dyn SettingsControl> {
        Box::new(source) as _
    }
}
