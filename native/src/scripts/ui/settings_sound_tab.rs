use godot::classes::Control;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, Rs, RsDyn, RsDynify};

use crate::resources::GameSettings;
use crate::scripts::ui::settings_option_range::SettingsOptionRange;
use crate::scripts::ui::tab_controller::{SettingsControl, SettingsTab};

#[derive(GodotScript, Debug)]
#[script(base = Control)]
pub(crate) struct SettingsSoundTab {
    #[export]
    pub master_bus: OnEditor<Rs<SettingsOptionRange>>,

    #[export]
    pub env_bus: OnEditor<Rs<SettingsOptionRange>>,

    #[export]
    pub music_bus: OnEditor<Rs<SettingsOptionRange>>,

    #[export]
    pub focus_start: OnEditor<Gd<Control>>,

    game_settings: Option<Gd<GameSettings>>,
}

#[godot_script_impl]
impl SettingsSoundTab {
    pub fn focus_start(&self) -> Gd<Control> {
        self.focus_start.clone()
    }

    pub fn set_game_settings(&mut self, settings: Gd<GameSettings>) {
        self.game_settings = Some(settings);
    }
}

impl SettingsTab for Rs<SettingsSoundTab> {
    fn focus_start(&mut self) -> Option<RsDyn<dyn SettingsControl>> {
        super::tab_controller::cast_settings_control(&ISettingsSoundTab::focus_start(self))
    }

    fn set_game_settings(&mut self, settings: Gd<GameSettings>) {
        ISettingsSoundTab::set_game_settings(self, settings);
    }
}

impl RsDynify<dyn SettingsTab> for SettingsSoundTab {
    fn coerce(source: Rs<Self>) -> Box<dyn SettingsTab> {
        Box::new(source) as _
    }
}
