use godot::builtin::math::ApproxEq;
use godot::builtin::Vector2i;
use godot::classes::display_server::VSyncMode;
use godot::classes::window::{self};
use godot::classes::{AudioServer, DisplayServer, Engine, ProjectSettings, SceneTree};
use godot::obj::{Gd, Singleton};
use godot::prelude::{godot_dyn, GodotClass};
use serde::{Deserialize, Serialize};

use crate::resources::toml_loader::SerializeTomlClass;
use crate::util::logger;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
struct AudioSettings {
    master: f32,
    env: f32,
    music: f32,
}

impl AudioSettings {
    fn apply(&self) {
        let mut audio_server = AudioServer::singleton();

        audio_server.set_bus_volume_linear(0, self.master);
        audio_server.set_bus_volume_linear(0, self.env);
        audio_server.set_bus_volume_linear(0, self.music);
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            master: 1.0,
            env: 1.0,
            music: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct VideoSettings {
    fullscreen: bool,
    vsync: bool,
    resolution: Vector2i,
}

impl VideoSettings {
    fn vsync_mode(project_settings: &Gd<ProjectSettings>) -> VSyncMode {
        project_settings
            .get_setting_with_override("display/window/vsync/vsync_mode")
            .to::<VSyncMode>()
    }

    fn window_mode(project_settings: &Gd<ProjectSettings>) -> window::Mode {
        project_settings
            .get_setting_with_override("display/window/size/mode")
            .to::<window::Mode>()
    }

    fn apply(&self) {
        let mut display_server = DisplayServer::singleton();
        let project_settings = ProjectSettings::singleton();

        let mut main_window = Engine::singleton()
            .get_main_loop()
            .unwrap()
            .cast::<SceneTree>()
            .get_root()
            .unwrap();

        logger::debug!("applying configured resolution: {}", self.resolution);
        main_window.set_content_scale_size(self.resolution);

        logger::debug!("applying fullscreen setting: {}", self.fullscreen);
        if self.fullscreen {
            main_window.set_mode(window::Mode::EXCLUSIVE_FULLSCREEN);
        } else {
            main_window.set_mode(window::Mode::WINDOWED);
        }

        let vsync_mode = if self.vsync {
            Self::vsync_mode(&project_settings)
        } else {
            VSyncMode::DISABLED
        };

        display_server
            .window_set_vsync_mode_ex(vsync_mode)
            .window_id(main_window.get_window_id())
            .done();
    }
}

impl Default for VideoSettings {
    fn default() -> Self {
        let project_settings = ProjectSettings::singleton();

        let fullscreen = VideoSettings::window_mode(&project_settings) != window::Mode::FULLSCREEN;

        let vsync = VideoSettings::vsync_mode(&project_settings) != VSyncMode::DISABLED;

        let res_x = project_settings
            .get_setting_with_override("display/window/size/viewport_width")
            .to::<i32>();
        let res_y = project_settings
            .get_setting_with_override("display/window/size/viewport_height")
            .to::<i32>();
        VideoSettings {
            fullscreen,
            vsync,
            resolution: Vector2i::new(res_x, res_y),
        }
    }
}

#[derive(Debug, GodotClass, Clone, Deserialize, Serialize, Default)]
#[class(base = Resource, init)]
#[serde(default)]
pub(crate) struct GameSettings {
    #[serde(skip)]
    dirty: bool,
    audio: AudioSettings,
    video: VideoSettings,
}

impl GameSettings {
    const PATH: &str = "user://settings.toml";

    pub fn apply(&mut self) {
        self.audio.apply();
        self.video.apply();
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_video_resolution(&mut self, resolution: Vector2i) {
        if self.video.resolution != resolution {
            self.dirty = true;
        }

        self.video.resolution = resolution;
    }

    pub fn set_video_fullscreen(&mut self, fullscreen: bool) {
        if self.video.fullscreen != fullscreen {
            self.dirty = true;
        }
        self.video.fullscreen = fullscreen;
    }

    pub fn set_video_vsync(&mut self, vsync: bool) {
        if self.video.vsync != vsync {
            self.dirty = true;
        }

        self.video.vsync = vsync;
    }

    pub fn set_audio_master_bus(&mut self, volume: f32) {
        if !self.audio.master.approx_eq(&volume) {
            self.dirty = true;
        }

        self.audio.master = volume;
    }

    pub fn set_audio_env_bus(&mut self, volume: f32) {
        if !self.audio.env.approx_eq(&volume) {
            self.dirty = true;
        }

        self.audio.env = volume;
    }

    pub fn set_audio_music_bus(&mut self, volume: f32) {
        if !self.audio.music.approx_eq(&volume) {
            self.dirty = true;
        }

        self.audio.music = volume;
    }

    pub fn load_or_new() -> Gd<Self> {
        let settings_resource = godot::tools::try_load::<GameSettings>(Self::PATH);

        let mut settings = match settings_resource {
            Ok(settings) => {
                logger::info!("loaded game settings: {:?}", *settings.bind());
                settings
            }

            Err(err) => {
                logger::warn!("failed to load game settings: {err}");
                let default_settings = GameSettings::default();

                let mut settings_res = Gd::from_object(default_settings);

                settings_res.set_path(Self::PATH);
                godot::tools::save(&settings_res, &settings_res.get_path());
                settings_res
            }
        };

        settings.bind_mut().apply();
        settings
    }
}

#[godot_dyn]
impl SerializeTomlClass for GameSettings {
    fn deserialize(
        &'_ mut self,
        der: toml::Table,
    ) -> Option<<toml::Table as serde::de::Deserializer<'_>>::Error> {
        let deserialized = match serde::Deserialize::deserialize(der) {
            Ok(de) => de,
            Err(err) => return Some(err),
        };

        logger::debug!("Deserialized game settings: {:?}", deserialized);

        *self = deserialized;
        None
    }

    fn serialize(&self) -> Result<toml::Table, toml::ser::Error> {
        toml::Table::try_from(self)
    }
}
