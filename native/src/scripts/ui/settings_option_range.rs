use godot::classes::{Control, Slider};
use godot::obj::Gd;
use godot::register::info::PropertyHint;
use godot::{builtin::GString, classes::Label};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptExportGroup};
use num::ToPrimitive;

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<Slider>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsOptionRange {
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
            self.set_step(value);

            if value > 0.0 {
                self.nodes
                    .value
                    .set_ticks(((self.max - self.min) / value).round().to_i32().unwrap() + 1);
                self.nodes.value.set_ticks_on_borders(true);
            }
        }

        self.step = value;
    }
}
