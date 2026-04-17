use godot::classes::{CheckButton, Control};
use godot::obj::Gd;
use godot::register::info::PropertyHint;
use godot::{builtin::GString, classes::Label};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptExportGroup};

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<CheckButton>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsOptionToggle {
    #[export(flatten)]
    pub nodes: ChildNodes,

    #[export]
    #[prop(set = Self::set_label)]
    pub label: GString,

    #[allow(clippy::manual_string_new)]
    #[export(custom(hint = PropertyHint::MULTILINE_TEXT, hint_string = ""))]
    pub description: GString,

    #[export]
    #[prop(set = Self::set_value)]
    pub value: bool,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsOptionToggle {
    pub fn _ready(&mut self) {
        self.set_value(self.value);
        self.set_label(self.label.clone());
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    fn set_value(&mut self, value: bool) {
        if self.base.is_node_ready() {
            self.nodes.value.set_pressed(value);
        }

        self.value = value;
    }
}
