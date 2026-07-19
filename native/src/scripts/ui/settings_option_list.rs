use godot::builtin::{Array, GString, Signal};
use godot::classes::{Control, Label, OptionButton};
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
    value: OnEditor<Gd<OptionButton>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control, tool)]
pub(crate) struct SettingsOptionList {
    #[signal]
    pub selection_changed: ScriptSignal<u32>,

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
        self.nodes
            .value
            .signals()
            .item_selected()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_selection_changed));
    }

    fn set_label(&mut self, value: GString) {
        if self.base.is_node_ready() {
            self.nodes.label.set_text(&value);
        }

        self.label = value;
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn set_values(&mut self, values: Array<GString>) {
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

    pub fn set_selected(&mut self, index: u32) {
        self.nodes.value.select(index.to_i32().unwrap());
    }

    pub fn get_selected(&mut self) -> u32 {
        self.nodes.value.get_selected().to_u32().unwrap()
    }

    pub fn on_focus(&mut self) {
        self.nodes.value.grab_focus();
    }

    pub fn on_selection_changed(&mut self, value: u32) {
        self.selected_index = value;
        self.selection_changed.emit(value);
    }

    pub fn selection_changed(&self) -> Signal {
        self.selection_changed.to_godot()
    }
}

impl SettingsControl for Rs<SettingsOptionList> {
    fn grab_focus(&mut self) {
        self.on_focus();
    }
}

impl RsDynify<dyn SettingsControl> for SettingsOptionList {
    fn coerce(source: godot_rust_script::Rs<Self>) -> Box<dyn SettingsControl> {
        Box::new(source) as Box<dyn SettingsControl>
    }
}
