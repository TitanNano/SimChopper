use godot::builtin::Signal;
use godot::classes::{AnimationPlayer, Button, CanvasItem, Control, InputEvent, TabBar};
use godot::meta::ToGodot;
use godot::obj::{Gd, WithUserSignals};
use godot_rust_script::{
    godot_script_impl, CastToScript, Context, GodotScript, OnEditor, RsDyn, ScriptExportGroup,
    ScriptSignal,
};

use crate::resources::{GameSettings, InputDevice};
use crate::script_callable;
use crate::scripts::ui::settings_option_input::SettingsOptionInput;
use crate::scripts::ui::settings_option_list::SettingsOptionList;
use crate::scripts::ui::settings_option_range::SettingsOptionRange;
use crate::scripts::ui::settings_option_toggle::SettingsOptionToggle;
use crate::scripts::ui::settings_sound_tab::SettingsSoundTab;
use crate::scripts::ui::settings_video_tab::SettingsVideoTab;
use crate::util::logger;

pub(crate) trait SettingsTab {
    fn focus_start(&mut self) -> Option<RsDyn<dyn SettingsControl>>;
    fn set_game_settings(&mut self, settings: Gd<GameSettings>);
}

pub(crate) trait SettingsControl {
    fn grab_focus(&mut self);
}

pub fn cast_settings_control(control: &Gd<Control>) -> Option<RsDyn<dyn SettingsControl>> {
    if let Ok(option) = control.try_to_script::<SettingsOptionList>() {
        return Some(option.into_trait());
    }

    if let Ok(option) = control.try_to_script::<SettingsOptionToggle>() {
        return Some(option.into_trait());
    }

    if let Ok(option) = control.try_to_script::<SettingsOptionRange>() {
        return Some(option.into_trait());
    }

    if let Ok(option) = control.try_to_script::<SettingsOptionInput>() {
        return Some(option.into_trait());
    }

    None
}

fn cast_settings_tab(tab: &Gd<Control>) -> Option<RsDyn<dyn SettingsTab>> {
    if let Ok(script) = tab.try_to_script::<SettingsVideoTab>() {
        Some(script.into_trait())
    } else if let Ok(script) = tab.try_to_script::<SettingsSoundTab>() {
        Some(script.into_trait())
    } else {
        None
    }
}

#[derive(ScriptExportGroup, Debug, Default)]
struct ChildNodes {
    tab_control: OnEditor<Gd<TabBar>>,
    tabs_container: OnEditor<Gd<Control>>,
    accept: OnEditor<Gd<Button>>,
    reset: OnEditor<Gd<Button>>,
    cancel: OnEditor<Gd<Button>>,
    animation_player: OnEditor<Gd<AnimationPlayer>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Control)]
pub struct TabController {
    #[export]
    pub input_device: OnEditor<Gd<InputDevice>>,

    #[export(flatten)]
    #[expect(private_interfaces)]
    pub nodes: ChildNodes,

    settings: Option<Gd<GameSettings>>,

    #[signal]
    pub settings_closed: ScriptSignal<()>,

    base: Gd<Control>,
}

#[godot_script_impl]
impl TabController {
    pub fn _ready(&mut self) {
        let tab_index = self.nodes.tab_control.get_current_tab();
        let Some(settings) = self.settings.clone() else {
            logger::error!("Settings must be assigned before adding the UI to the tree!");
            return;
        };

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

        drop(input_device);

        self.nodes
            .accept
            .signals()
            .pressed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_settings_accept));

        self.nodes
            .reset
            .signals()
            .pressed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_settings_reset));

        self.nodes
            .cancel
            .signals()
            .pressed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_settings_cancel));

        for child in self.nodes.tabs_container.get_children().iter_shared() {
            let Ok(control) = child.try_cast::<Control>() else {
                continue;
            };

            let Some(mut tab) = cast_settings_tab(&control) else {
                continue;
            };

            tab.set_game_settings(settings.clone());
        }
    }

    pub fn _process(&mut self, _delta: f32) {
        let dirty_settings = self
            .settings
            .as_ref()
            .is_some_and(|settings| settings.bind().is_dirty());

        self.nodes.accept.set_disabled(!dirty_settings);
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

        let mut tab_container = tab_container.try_cast::<Control>().unwrap();

        tab_container.set_visible(true);

        let Some(mut settings_tab) = cast_settings_tab(&tab_container) else {
            logger::warn!(
                "Settings tab script {:?} is not one of the know tab types.",
                tab_container
                    .get_script()
                    .map(|script| script.get_global_name())
            );
            return;
        };

        if let Some(mut control) = settings_tab.focus_start() {
            control.grab_focus();
        }
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

    fn close(&mut self) {
        self.nodes.animation_player.play_ex().name("exit").done();
    }

    pub fn finalize_close(&mut self) {
        self.base.queue_free();
        self.settings_closed.emit(());
    }

    pub fn on_settings_accept(&mut self) {
        if let Some(settings) = &mut self.settings {
            settings.bind_mut().apply();
        } else {
            logger::warn!("No settings assigned to settings UI!");
        }

        self.close();
    }

    #[expect(clippy::unused_self)]
    pub fn on_settings_reset(&self) {
        logger::info!("reset pressed!");
    }

    pub fn on_settings_cancel(&mut self) {
        self.close();
    }

    pub fn settings_closed(&self) -> Signal {
        self.settings_closed.to_godot()
    }

    pub fn set_settings(&mut self, settings: Gd<GameSettings>) {
        self.settings = Some(settings);
    }
}
