use godot::builtin::{vslice, GString, Signal};
use godot::classes::{CheckButton, Control, Label};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot::register::info::PropertyHint;
use godot_rust_script::{
    godot_script_impl, GodotScript, OnEditor, Rs, RsDynify, ScriptExportGroup, ScriptSignal,
};

use crate::script_callable;
use crate::scripts::ui::tab_controller::SettingsControl;

#[derive(ScriptExportGroup, Debug, Default)]
pub(crate) struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<CheckButton>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
pub(crate) struct SettingsOptionToggle {
    #[signal]
    pub value_changed: ScriptSignal<bool>,

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
        self.nodes
            .value
            .signals()
            .toggled()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_value_changed));
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    pub fn set_value(&mut self, value: bool) {
        if self.base.is_node_ready() {
            self.nodes
                .value
                .call_deferred("set_pressed", vslice![value]);
        }

        self.value = value;
    }

    pub fn grab_focus(&mut self) {
        self.nodes.value.grab_focus();
    }

    pub fn on_value_changed(&mut self, value: bool) {
        self.value = value;
        self.value_changed.emit(value);
    }

    pub fn value_changed(&self) -> Signal {
        self.value_changed.to_godot()
    }
}

impl SettingsControl for Rs<SettingsOptionToggle> {
    fn grab_focus(&mut self) {
        ISettingsOptionToggle::grab_focus(self);
    }
}

impl RsDynify<dyn SettingsControl> for SettingsOptionToggle {
    fn coerce(source: Rs<Self>) -> Box<dyn SettingsControl> {
        Box::new(source) as Box<dyn SettingsControl>
    }
}
