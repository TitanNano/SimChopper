use godot::classes::{CanvasItem, Control, InputEvent, TabBar};
use godot::obj::{Gd, WithUserSignals};
use godot_rust_script::{godot_script_impl, Context, GodotScript, OnEditor, ScriptExportGroup};

use crate::resources::InputDevice;
use crate::script_callable;
use crate::util::logger;

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    tab_control: OnEditor<Gd<TabBar>>,
    tabs_container: OnEditor<Gd<Control>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control)]
struct TabController {
    #[export]
    pub input_device: OnEditor<Gd<InputDevice>>,

    #[export(flatten)]
    pub nodes: ChildNodes,

    base: Gd<Control>,
}

#[godot_script_impl]
impl TabController {
    pub fn _ready(&mut self) {
        let tab_index = self.nodes.tab_control.get_current_tab();

        self.on_tab_change(tab_index);
        self.nodes
            .tab_control
            .signals()
            .tab_changed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_tab_change));

        let on_next_tab = script_callable!(self, Self::on_next_tab);
        let on_prev_tab = script_callable!(self, Self::on_prev_tab);
        let mut input_device = self.input_device.bind_mut();

        input_device
            .signals()
            .ui_tab_next()
            .to_untyped()
            .connect(&on_next_tab);

        input_device
            .signals()
            .ui_tab_prev()
            .to_untyped()
            .connect(&on_prev_tab);
    }

    pub fn on_tab_change(&mut self, tab_index: i32) {
        for tab in self.nodes.tabs_container.get_children().iter_shared() {
            let Ok(mut tab) = tab.try_cast::<CanvasItem>() else {
                continue;
            };

            tab.set_visible(false);
        }

        if tab_index < 0 {
            return;
        }

        let Some(tab_container) = self.nodes.tabs_container.get_child(tab_index) else {
            logger::error!("Tab index out of bounds!");
            return;
        };

        tab_container
            .try_cast::<CanvasItem>()
            .unwrap()
            .set_visible(true);
    }

    pub fn _unhandled_input(&mut self, event: Gd<InputEvent>, mut context: Context<Self>) {
        let mut input_device = self.input_device.clone();

        context.reentrant_scope(self, |_base| {
            input_device.bind_mut().capture(event);
        });
    }

    pub fn on_next_tab(&mut self, pressed: bool, mut context: Context<Self>) {
        if !pressed {
            return;
        }

        let mut tab_control = self.nodes.tab_control.clone();

        context.reentrant_scope(self, |_base| {
            tab_control.select_next_available();
        });
    }

    pub fn on_prev_tab(&mut self, pressed: bool, mut context: Context<Self>) {
        if !pressed {
            return;
        }

        let mut tab_control = self.nodes.tab_control.clone();

        context.reentrant_scope(self, |_base| {
            tab_control.select_previous_available();
        });
    }
}
