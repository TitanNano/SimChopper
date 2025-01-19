use godot::builtin::math::FloatExt;
use godot::builtin::Vector3;
use godot::classes::{light_3d, DirectionalLight3D, Node3D, Time};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
pub struct SolarSetup {
    /// Reference to the sun child node.
    #[export]
    pub sun: Option<Gd<DirectionalLight3D>>,

    /// Reference to the moon child node.
    #[export]
    pub moon: Option<Gd<DirectionalLight3D>>,

    /// duration from sun rise to sun set in minutes
    #[export(range(min = 1.0, max = 120.0, step = 1.0))]
    pub day_length: u64,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl SolarSetup {
    pub fn _physics_process(&mut self, _delta: f64) {
        let time = self.get_time();
        let day_length = self.day_length_ms();
        let sun_pos = time as f32 * (360.0 / (day_length * 2) as f32);

        let Some(ref mut sun) = self.sun else {
            logger::error!("no sun is assigned to solar setup!");
            logger::error!("node path: {}", self.base.get_path());

            return;
        };

        let Some(ref mut moon) = self.moon else {
            logger::error!("no moon is assinged to solar setup!");
            logger::error!("node path: {}", self.base.get_path());
            return;
        };

        self.base
            .set_rotation_degrees(Vector3::new(sun_pos, 0.0, 0.0));

        let sun_visible = sun_pos < 190.0;
        let sun_zenit_distance = ((sun_pos - 90.0) / 90.0).abs().clamp(0.0, 1.0);

        sun.set_param(
            light_3d::Param::ENERGY,
            if !sun_visible { 0.0 } else { 1.0 },
        );
        sun.set_shadow(sun_visible);

        if sun_visible {
            sun.set_param(
                light_3d::Param::INTENSITY,
                100000.0.lerp(400.0, sun_zenit_distance),
            );
            sun.set_temperature((5500.0).lerp(1850.0, sun_zenit_distance));
        }

        moon.set_param(
            light_3d::Param::ENERGY,
            if sun_pos > 180.0 { 1.0 } else { 0.0 },
        );
        moon.set_shadow(sun_pos > 180.0);
    }

    /// Day length (sunrise to sunset) in ms.
    fn day_length_ms(&self) -> u64 {
        self.day_length * 60 * 1000
    }

    /// Get the current game time in ms.
    pub fn get_time(&self) -> u64 {
        let day_length = self.day_length_ms();

        Time::singleton().get_ticks_msec() % (day_length * 2)
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
        self.get_time() as f64 / (self.day_length_ms() as f64 * 2.0 / 24.0)
    }
}
