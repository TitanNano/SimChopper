use std::f32;

use godot::builtin::math::FloatExt;
use godot::builtin::Vector3;
use godot::classes::class_macros::private::virtuals::Xrvrs::math::ApproxEq;
use godot::classes::{light_3d, DirectionalLight3D, Node3D, Performance, Time, VoxelGiData};
use godot::obj::{Gd, Singleton as _};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, ScriptSignal};
use num::ToPrimitive;

use crate::script_callable;
use crate::util::Uf32;

const UPDATE_INTERVAL: f32 = 1.0 / 4.0;
const MOON_MIN: f32 = 177.0;
const MOON_FULL: f32 = 183.0;
const SUN_DOWN_END: f32 = 192.0;
const SUN_RISE_START: f32 = 349.0;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
pub struct SolarSetup {
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
    pub day_length: Uf32,

    // The VoxelGI parameters that are used by the scene.
    #[export]
    pub voxel_gi_data: OnEditor<Gd<VoxelGiData>>,

    sun_pos: f32,
    sun_zenit_distance: f32,

    update_delay: f32,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl SolarSetup {
    const SUN_LUX_MIN: f32 = 400.0;
    const SUN_LUX_MAX: f32 = 73_728.0;
    const GI_CONTRIBUTION: f32 = 0.2;

    pub fn _ready(&mut self) {
        #[cfg(debug_assertions)]
        self.debug_monitors();
    }

    #[cfg(debug_assertions)]
    pub fn debug_monitors(&self) {
        let mut performance = Performance::singleton();

        performance.add_custom_monitor(
            "Game/ClockHour",
            &script_callable!(self, Self::get_ingame_clock_h),
        );
        performance.add_custom_monitor(
            "Game/ClockMinute",
            &script_callable!(self, Self::get_ingame_clock_m),
        );
        performance.add_custom_monitor(
            "Light/Env",
            &script_callable!(self, Self::environment_brightness),
        );
        performance.add_custom_monitor("Light/Sun", &script_callable!(self, Self::sun_brightness));
        performance
            .add_custom_monitor("Light/Moon", &script_callable!(self, Self::moon_brightness));
        performance.add_custom_monitor("Light/Sky", &script_callable!(self, Self::sky_brightness));

        performance.add_custom_monitor("Sun/Pos", &script_callable!(self, Self::sun_pos));
        performance.add_custom_monitor(
            "Sun/Zenit_Distance",
            &script_callable!(self, Self::sun_zenite_distance),
        );
    }

    pub fn _physics_process(&mut self, delta: f32) {
        let update_delay = self.update_delay + delta;

        self.update_delay = update_delay % UPDATE_INTERVAL;

        // only update when we cross the delay threshold
        if update_delay.approx_eq(&self.update_delay) {
            return;
        }

        let time = self.get_time();
        let day_length = self.day_length_ms();

        let sun_pos = time.into_f32() * (360.0 / (day_length.into_f32() * 2.0));

        // gradually reduce sun energy when it sinks under the horizon.
        let sun_horizon_range = {
            let sun_down = 1.0 - ((sun_pos - 180.0) / (SUN_DOWN_END - 180.0)).clamp(0.0, 1.0);
            let sun_rise = ((sun_pos - SUN_RISE_START) / (360.0 - SUN_RISE_START)).clamp(0.0, 1.0);

            sun_down + sun_rise
        };
        let sun_visible = sun_pos < 190.0;
        let sun_zenit_distance = ((sun_pos - 90.0) / 90.0).abs().clamp(0.0, 1.0);
        let sun_energy = sun_horizon_range;

        let sun_lux = Self::SUN_LUX_MAX.lerp(Self::SUN_LUX_MIN, sun_zenit_distance);

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

        let moon_visible = sun_pos > MOON_MIN;
        let moon_horizon_range = ((sun_pos - MOON_MIN) / (MOON_FULL - MOON_MIN)).clamp(0.0, 1.0);
        let moon_energy = moon_horizon_range;

        self.moon.set_param(light_3d::Param::ENERGY, moon_energy);
        self.moon.set_shadow(moon_visible);

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
    }

    /// Day length (sunrise to sunset) in ms.
    fn day_length_ms(&self) -> Uf32 {
        self.day_length * Uf32::new(60) * Uf32::new(1000)
    }

    /// Get the current game time in ms.
    pub fn get_time(&self) -> Uf32 {
        let day_length = self.day_length_ms();
        // start game at 30% of the day.
        let base_offset = (day_length / Uf32::new(10)) * Uf32::new(3);

        (base_offset
            + Uf32::new(
                Time::singleton()
                    .get_ticks_msec()
                    .try_into()
                    .expect("lets pray the game ticks fit into u32::MAX"),
            ))
            % (day_length * Uf32::new(2))
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
        (self.get_time().into_f64() / (self.day_length_ms().into_f64() * 2.0 / 24.0)) % 24.0
    }

    pub fn get_ingame_clock_h(&self) -> u32 {
        // sunrise is at 6 am so an in-game time of 0 hours is equal to 6 am.
        (self
            .get_ingame_time_h()
            .floor()
            .to_u32()
            .expect("time should alawys be positive and fit")
            + 6)
            % 24
    }

    pub fn get_ingame_clock_m(&self) -> Uf32 {
        let hours = self.get_ingame_time_h().floor();

        Uf32::new(
            (self.get_ingame_time_m().floor() - hours * 60.0)
                .to_u32()
                .expect("time should always be positive and fit"),
        )
    }

    pub fn sun_pos(&self) -> f32 {
        self.sun_pos
    }

    pub fn sun_zenite_distance(&self) -> f32 {
        self.sun_zenit_distance * 100.0
    }

    /// Total brightness of the sun in lux.
    #[inline]
    pub fn sun_brightness(&self) -> f32 {
        let voxel_gi_energy = self.voxel_gi_data.get_energy();

        let sun_brightness = self.sun.get_param(light_3d::Param::INTENSITY)
            * self.sun.get_param(light_3d::Param::ENERGY);

        let sun_gi = voxel_gi_energy * sun_brightness * Self::GI_CONTRIBUTION;

        sun_brightness + sun_gi
    }

    /// Total brightness of the moon in lux.
    #[inline]
    pub fn moon_brightness(&self) -> f32 {
        let voxel_gi_energy = self.voxel_gi_data.get_energy();

        let moon_brightness = self.moon.get_param(light_3d::Param::INTENSITY)
            * self.moon.get_param(light_3d::Param::ENERGY);

        let moon_gi = voxel_gi_energy * moon_brightness * Self::GI_CONTRIBUTION;

        moon_brightness + moon_gi
    }

    /// Total brigtness of the sky in lux.
    #[inline]
    pub fn sky_brightness(&self) -> f32 {
        let environment = self
            .base
            .get_world_3d()
            .expect("solar setup must be part of a scene")
            .get_environment()
            .expect("scene must have an environment");

        // boost the background light because the reflection of the sun is causing a lot of additional indirect light at narrow angles.
        let sun_down = self.sun_pos > 180.0 && self.sun_pos < SUN_DOWN_END;
        let sun_up = self.sun_pos > SUN_RISE_START;

        let boost = if sun_down || sun_up { 100.0 } else { 0.0 };

        environment.get_bg_intensity() * f32::consts::PI + boost
    }

    /// Total brigtness of the sky and all visible solar bodys in lux.
    ///
    /// This is used to get an aproximation of the overal scene brightness.
    pub fn environment_brightness(&self) -> f32 {
        self.sun_brightness() + self.moon_brightness() + self.sky_brightness()
    }
}
