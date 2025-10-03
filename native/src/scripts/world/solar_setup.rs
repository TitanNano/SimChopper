use godot::builtin::math::FloatExt;
use godot::builtin::Vector3;
use godot::classes::{light_3d, DirectionalLight3D, Node3D, Performance, Time};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptSignal};

use crate::script_callable;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
pub struct SolarSetup {
    #[signal("yes")]
    pub sun_visible: ScriptSignal<bool>,

    #[signal("brightness")]
    pub sky_brightness: ScriptSignal<f64>,

    /// Reference to the sun child node.
    #[export]
    pub sun: OnEditor<Gd<DirectionalLight3D>>,

    /// Reference to the moon child node.
    #[export]
    pub moon: OnEditor<Gd<DirectionalLight3D>>,

    /// duration from sun rise to sun set in minutes
    #[export(range(min = 1.0, max = 120.0, step = 1.0))]
    pub day_length: u32,

    /// The minimum brightness of the sky.
    #[export(range(min = 1.0, max = 30_000.0, step = 1.0))]
    pub sky_min_brightness: f32,

    /// The maximum brightness of the sky.
    #[export(range(min = 1.0, max = 30_000.0, step = 1.0))]
    pub sky_max_brightness: f32,

    sdfgi_enabled: bool,

    sun_pos: f32,
    sun_zenit_distance: f32,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl SolarSetup {
    const SUN_LUX_MIN: f32 = 400.0;
    const SUN_LUX_MAX: f32 = 81920.0;

    pub fn _ready(&mut self) {
        let env = self
            .base
            .get_world_3d()
            .expect("solar setup must be part of the scene tree")
            .get_environment()
            .expect("there must be an environment");

        self.sdfgi_enabled = env.is_sdfgi_enabled();
        Performance::singleton().add_custom_monitor(
            "Game/ClockHour",
            &script_callable!(self, Self::get_ingame_clock_h),
        );
        Performance::singleton().add_custom_monitor(
            "Game/ClockMinute",
            &script_callable!(self, Self::get_ingame_clock_m),
        );
    }

    pub fn _physics_process(&mut self, _delta: f64) {
        let time = self.get_time();
        let day_length = self.day_length_ms();

        let sun_pos = time as f32 * (360.0 / (day_length * 2) as f32);
        let sun_visible = sun_pos < 190.0;
        let sun_zenit_distance = ((sun_pos - 90.0) / 90.0).abs().clamp(0.0, 1.0);
        let sun_energy = if !sun_visible { 0.0 } else { 1.0 };
        let sun_lux = Self::SUN_LUX_MAX.lerp(Self::SUN_LUX_MIN, sun_zenit_distance);

        let moon_energy = if sun_pos > 180.0 { 1.0 } else { 0.0 };

        self.sun_pos = sun_pos;
        self.sun_zenit_distance = sun_zenit_distance;

        self.base
            .set_rotation_degrees(Vector3::new(sun_pos, 0.0, 0.0));

        self.sun.set_param(light_3d::Param::ENERGY, sun_energy);
        self.sun.set_shadow(sun_visible);

        if sun_visible {
            self.sun.set_param(light_3d::Param::INTENSITY, sun_lux);
            self.sun
                .set_temperature((5500.0).lerp(1850.0, sun_zenit_distance));
        }

        let mut env = self
            .base
            .get_world_3d()
            .expect("solar setup must be part of the scene tree")
            .get_environment()
            .expect("there must be an environment");

        env.set_bg_intensity(
            (sun_lux * sun_energy + self.moon.get_param(light_3d::Param::INTENSITY) * moon_energy)
                * 0.2,
        );

        if self.sdfgi_enabled {
            // disable SDFGI during night time. It's causing too much fluctuation in the overall brightness of the scene under low light conditions.
            env.set_sdfgi_enabled(sun_pos < 180.0);
        }

        self.moon.set_param(light_3d::Param::ENERGY, moon_energy);
        self.moon.set_shadow(sun_pos > 180.0);
    }

    /// Day length (sunrise to sunset) in ms.
    fn day_length_ms(&self) -> u64 {
        self.day_length as u64 * 60 * 1000
    }

    /// Get the current game time in ms.
    pub fn get_time(&self) -> u64 {
        let day_length = self.day_length_ms();
        // start game at 30% of the day.
        let base_offset = (day_length / 10) * 3;

        (base_offset + Time::singleton().get_ticks_msec()) % (day_length * 2)
    }

    // get the current in-game time in seconds since sunrise.
    pub fn get_ingame_time_s(&self) -> f64 {
        self.get_ingame_time_m() * 60.0
    }

    // get the current in-game time in minutes since sunrise.
    pub fn get_ingame_time_m(&self) -> f64 {
        self.get_ingame_time_h() * 60.0
    }

    // get the current in-game time in hours since sunrise.
    pub fn get_ingame_time_h(&self) -> f64 {
        (self.get_time() as f64 / (self.day_length_ms() as f64 * 2.0 / 24.0)) % 24.0
    }

    pub fn get_ingame_clock_h(&self) -> u32 {
        // sunrise is at 6 am so an in-game time of 0 hours is equal to 6 am.
        (self.get_ingame_time_h().floor() as u32 + 6) % 24
    }

    pub fn get_ingame_clock_m(&self) -> u32 {
        let hours = self.get_ingame_time_h().floor();

        (self.get_ingame_time_m().floor() - hours * 60.0) as u32
    }

    pub fn sun_pos(&self) -> f32 {
        self.sun_pos
    }

    pub fn sun_zenite_distance(&self) -> f32 {
        self.sun_zenit_distance
    }

    /// Total brigtness of the sky and all visible solar bodys.
    ///
    /// This is used to get an aproximation of the overal scene brightness.
    pub fn environment_brightness(&self) -> f32 {
        let sun_brightness = self.sun.get_param(light_3d::Param::INTENSITY)
            * self.sun.get_param(light_3d::Param::ENERGY);

        let moon_brightness = self.moon.get_param(light_3d::Param::INTENSITY)
            * self.moon.get_param(light_3d::Param::ENERGY);

        let environment = self
            .base
            .get_world_3d()
            .expect("solar setup must be part of a scene")
            .get_environment()
            .expect("scene must have an environment");
        let sky_brightness = environment.get_bg_intensity();

        sun_brightness + moon_brightness + sky_brightness
    }
}
