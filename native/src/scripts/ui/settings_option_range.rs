use godot::builtin::{GString, Signal};
use godot::classes::{Control, Label, Slider};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot::register::info::PropertyHint;
use godot_rust_script::{
    godot_script_impl, GodotScript, OnEditor, Rs, RsDynify, ScriptExportGroup, ScriptSignal,
};
use num::ToPrimitive;

use crate::script_callable;
use crate::scripts::ui::tab_controller::SettingsControl;

#[derive(ScriptExportGroup, Debug, Default)]
pub(crate) struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<Slider>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
pub struct SettingsOptionRange {
    #[signal]
    pub value_changed: ScriptSignal<f64>,

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
    pub value: f64,

    #[export]
    #[prop(set = Self::set_min)]
    pub min: f64,

    #[export]
    #[prop(set = Self::set_max)]
    pub max: f64,

    #[export]
    #[prop(set = Self::set_step)]
    pub step: f64,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsOptionRange {
    pub fn _ready(&mut self) {
        self.set_value(self.value);
        self.set_label(self.label.clone());
        self.set_step(self.step);

        self.nodes
            .value
            .signals()
            .value_changed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_value_changed));
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    fn set_value(&mut self, value: f64) {
        if self.base.is_node_ready() {
            self.nodes.value.set_value(value);
        }

        self.value = value;
    }

    fn set_min(&mut self, value: f64) {
        if self.base.is_node_ready() {
            self.nodes.value.set_min(value);
        }

        self.min = value;
    }

    fn set_max(&mut self, value: f64) {
        if self.base.is_node_ready() {
            self.nodes.value.set_max(value);
        }

        self.max = value;
    }

    fn set_step(&mut self, value: f64) {
        if self.base.is_node_ready() {
            self.nodes.value.set_step(value);

            if value > 1.0 {
                self.nodes
                    .value
                    .set_ticks(((self.max - self.min) / value).round().to_i32().unwrap() + 1);
                self.nodes.value.set_ticks_on_borders(true);
            } else {
                self.nodes.value.set_ticks_on_borders(false);
                self.nodes.value.set_ticks(0);
            }
        }

        self.step = value;
    }

    pub fn grab_focus(&mut self) {
        self.nodes.value.grab_focus();
    }

    pub fn on_value_changed(&mut self, value: f64) {
        self.value = value;
        self.value_changed.emit(value);
    }

    pub fn value_changed(&self) -> Signal {
        self.value_changed.to_godot()
    }
}

impl SettingsControl for Rs<SettingsOptionRange> {
    fn grab_focus(&mut self) {
        ISettingsOptionRange::grab_focus(self);
    }
}

impl RsDynify<dyn SettingsControl> for SettingsOptionRange {
    fn coerce(source: Rs<Self>) -> Box<dyn SettingsControl> {
        Box::new(source) as Box<dyn SettingsControl>
    }
}
