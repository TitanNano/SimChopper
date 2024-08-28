use godot::builtin::Vector3;
use godot::engine::{light_3d, DirectionalLight3D, Node3D, Time};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct SolarSetup {
    /// Reference to the sun child node.
    #[export]
    pub sun: Option<Gd<DirectionalLight3D>>,

    /// Reference to the moon child node.
    #[export]
    pub moon: Option<Gd<DirectionalLight3D>>,

    cached_moon_brightness: f32,

    // duration from sun rise to sun set in minutes
    #[export(range(min = 1.0, max = 120.0, step = 1.0))]
    pub day_length: u64,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl SolarSetup {
    pub fn _ready(&mut self) {
        let Some(ref mut moon) = self.moon else {
            logger::error!("no moon is assinged to solar setup!");
            logger::error!("node path: {}", self.base.get_path());
            return;
        };

        self.cached_moon_brightness = moon.get_param(light_3d::Param::ENERGY);
    }

    pub fn _physics_process(&mut self, _delta: f64) {
        let day_length = self.day_length * 60 * 1000;
        let time = Time::singleton().get_ticks_msec() % (day_length * 2);

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

        sun.set_param(
            light_3d::Param::ENERGY,
            if sun_pos > 190.0 { 0.0 } else { 1.0 },
        );
        sun.set_shadow(sun_pos < 190.0);

        moon.set_param(
            light_3d::Param::ENERGY,
            if sun_pos > 180.0 {
                self.cached_moon_brightness
            } else {
                0.0
            },
        );
        moon.set_shadow(sun_pos > 180.0);
    }
}
