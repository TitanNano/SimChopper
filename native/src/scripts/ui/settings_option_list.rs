use godot::builtin::{Array, GString};
use godot::classes::{Control, Label, OptionButton};
use godot::obj::Gd;
use godot::register::info::PropertyHint;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptExportGroup};
use num::ToPrimitive;

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    label: OnEditor<Gd<Label>>,
    value: OnEditor<Gd<OptionButton>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
struct SettingsOptionList {
    /// Internal child node references.
    #[export(flatten)]
    pub nodes: ChildNodes,

    /// Label of the settings option.
    #[export]
    #[prop(set = Self::set_label)]
    pub label: GString,

    /// Description of the settings option.
    ///
    /// This text will be displayed on the right-hand side of the settings screen.
    #[allow(clippy::manual_string_new)]
    #[export(custom(hint = PropertyHint::MULTILINE_TEXT, hint_string = ""))]
    pub description: GString,

    /// Possible values for this settings option.
    #[export]
    #[prop(set = Self::set_values)]
    pub values: Array<GString>,

    /// The index of the currently selected value.
    #[export]
    pub selected_index: u32,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsOptionList {
    pub fn _ready(&mut self) {
        self.set_values(self.values.clone());
        self.set_label(self.label.clone());
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    #[allow(clippy::needless_pass_by_value)]
    fn set_values(&mut self, values: Array<GString>) {
        if self.base.is_node_ready() {
            self.nodes
                .value
                .set_item_count(values.len().to_i32().unwrap());

            for (idx, value) in values.iter_shared().enumerate() {
                self.nodes
                    .value
                    .set_item_text(idx.to_i32().unwrap(), &value);
            }
        }

        self.values = values;
    }
}
