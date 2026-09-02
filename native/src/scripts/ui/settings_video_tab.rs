use std::collections::HashSet;

use godot::builtin::{GString, Vector2i};
use godot::classes::display_server::VSyncMode;
use godot::classes::{window, Control, DisplayServer};
use godot::obj::{Gd, Singleton};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, Rs, RsDyn, RsDynify};
use itertools::Itertools;
use num::ToPrimitive;

use crate::resources::GameSettings;
use crate::script_callable;
use crate::scripts::ui::settings_option_list::{ISettingsOptionList, SettingsOptionList};
use crate::scripts::ui::settings_option_toggle::{ISettingsOptionToggle, SettingsOptionToggle};
use crate::scripts::ui::tab_controller::{SettingsControl, SettingsTab};
use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Control)]
pub(crate) struct SettingsVideoTab {
    #[export]
    pub focus_start: OnEditor<Gd<Control>>,

    #[export]
    pub resolution: OnEditor<Rs<SettingsOptionList>>,

    #[export]
    pub fullscreen: OnEditor<Rs<SettingsOptionToggle>>,

    #[export]
    pub vsync: OnEditor<Rs<SettingsOptionToggle>>,

    resolution_options: Vec<Vector2i>,

    game_settings: Option<Gd<GameSettings>>,

    base: Gd<Control>,
}

#[godot_script_impl]
impl SettingsVideoTab {
    pub fn _ready(&mut self) {
        self.setup_resolution();
        self.setup_fullscreen();
    }

    pub fn focus_start(&self) -> Gd<Control> {
        self.focus_start.clone()
    }

    fn setup_resolution(&mut self) {
        let mut native_resolution = DisplayServer::singleton().screen_get_size();

        #[cfg(target_os = "macos")]
        {
            // Godot does not yet correctly support the MacBook display cutout. Temporary hardcoded inset until PR is merged:
            // https://github.com/godotengine/godot/pull/119196
            native_resolution.y -= 76;
        }

        logger::debug!(
            "display safe area: {}",
            DisplayServer::singleton().get_display_safe_area()
        );
        logger::debug!(
            "display cutouts: {}",
            DisplayServer::singleton().get_display_cutouts()
        );

        let current_resolution = self.base.get_window().unwrap().get_content_scale_size();
        let aspect_ratio = f64::from(native_resolution.x) / f64::from(native_resolution.y);

        let mut resolution_set = HashSet::with_capacity(4);
        resolution_set.insert(native_resolution);
        resolution_set.insert(native_resolution * 2 / 3);
        resolution_set.insert(native_resolution / 2);
        resolution_set.insert(native_resolution * 2 / 5);
        resolution_set.insert(native_resolution / 3);
        resolution_set.insert(Vector2i::new(1920, 1080));
        resolution_set.insert(Vector2i::new(
            (1080.0 * aspect_ratio).round().to_i32().unwrap(),
            1080,
        ));
        resolution_set.insert(Vector2i::new(1280, 720));
        resolution_set.insert(Vector2i::new(
            (720.0 * aspect_ratio).round().to_i32().unwrap(),
            720,
        ));

        let option_list: Vec<Vector2i> = resolution_set.into_iter().sorted().rev().collect();
        let option_labels = option_list
            .iter()
            .map(|res| GString::from(&format!("{}x{}", res.x, res.y)))
            .collect();
        let current = option_list
            .iter()
            .find_position(|item| **item == current_resolution)
            .map_or_else(|| option_list.len() - 1, |(index, _)| index);

        logger::debug!("current resolution : {}", current_resolution);

        self.resolution.set_values(option_labels);
        self.resolution.set_selected(current.to_u32().unwrap());
        self.resolution_options = option_list;
        self.resolution
            .selection_changed()
            .connect(&script_callable!(self, Self::on_resolution_changed));
    }

    pub fn on_resolution_changed(&mut self, index: u32) {
        let Some(game_settings) = self.game_settings.as_mut() else {
            logger::warn!("Resolution changed but game settings are unset!");
            return;
        };

        let Some(res) = self.resolution_options.get(index as usize) else {
            logger::error!("resolution control selected unknown option {}", index);
            return;
        };

        logger::debug!("setting new resolution: {}", res);

        game_settings.bind_mut().set_video_resolution(*res);
    }

    fn setup_fullscreen(&mut self) {
        let is_fullscreen = self.base.get_window().unwrap().get_mode() == window::Mode::FULLSCREEN;

        self.fullscreen.set_value(is_fullscreen);
        self.fullscreen
            .value_changed()
            .connect(&script_callable!(self, Self::on_fullscreen_changed));
    }

    pub fn on_fullscreen_changed(&mut self, value: bool) {
        let Some(game_settings) = self.game_settings.as_mut() else {
            return;
        };
        logger::info!("vsync changed");

        game_settings.bind_mut().set_video_fullscreen(value);
    }

    pub fn setup_vsync(&mut self) {
        let use_vsync = DisplayServer::singleton().window_get_vsync_mode() != VSyncMode::DISABLED;

        self.vsync.set_value(use_vsync);
        self.vsync
            .value_changed()
            .connect(&script_callable!(self, Self::on_vsync_changed));
    }

    pub fn on_vsync_changed(&mut self, value: bool) {
        let Some(game_settings) = self.game_settings.as_mut() else {
            return;
        };

        logger::info!("vsync changed");

        game_settings.bind_mut().set_video_vsync(value);
    }

    pub fn set_game_settings(&mut self, settings: Gd<GameSettings>) {
        self.game_settings = Some(settings);
    }
}

impl SettingsTab for Rs<SettingsVideoTab> {
    fn focus_start(&mut self) -> Option<RsDyn<dyn SettingsControl>> {
        super::tab_controller::cast_settings_control(&ISettingsVideoTab::focus_start(self))
    }

    fn set_game_settings(&mut self, settings: Gd<GameSettings>) {
        ISettingsVideoTab::set_game_settings(self, settings);
    }
}

impl RsDynify<dyn SettingsTab> for SettingsVideoTab {
    fn coerce(source: Rs<Self>) -> Box<dyn SettingsTab> {
        Box::new(source) as _
    }
}
