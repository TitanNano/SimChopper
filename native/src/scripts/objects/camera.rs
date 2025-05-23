use godot::classes::Camera3D;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, RsRef};

use crate::{
    scripts::world::solar_setup::{ISolarSetup, SolarSetup},
    warn,
};

#[derive(GodotScript, Debug)]
#[script(base = Camera3D)]
struct Camera {
    #[export]
    pub solar_setup: Option<RsRef<SolarSetup>>,

    base: Gd<Camera3D>,
}

#[godot_script_impl]
impl Camera {
    const MID_DAY_EXPOSURE: f32 = 10.0;
    const DAWN_EXPOSURE: f32 = 30.0;
    const NIGHT_EXPOSURE: f32 = 1000.0;

    pub fn _ready(&mut self) {}

    pub fn _process(&mut self, _delta: f64) {
        let Some(solar_setup) = self.solar_setup.as_mut() else {
            warn!("Solar setup is not available!");
            return;
        };

        let Some(mut attributes) = self.base.get_attributes() else {
            warn!("Camera has no attributes!");
            return;
        };

        let game_time = solar_setup.get_ingame_time_h();

        let exposure = match game_time {
            0.0..6.0 => {
                Self::DAWN_EXPOSURE
                    - ((Self::DAWN_EXPOSURE - Self::MID_DAY_EXPOSURE) / 6.0) * game_time as f32
            }
            6.0..12.0 => {
                Self::MID_DAY_EXPOSURE
                    + ((Self::DAWN_EXPOSURE - Self::MID_DAY_EXPOSURE) / 6.0)
                        * (game_time as f32 - 6.0)
            }
            12.0..12.5 => {
                Self::DAWN_EXPOSURE
                    + ((Self::NIGHT_EXPOSURE - Self::DAWN_EXPOSURE) / 6.0)
                        * (game_time as f32 - 12.0)
            }
            12.5..23.5 => Self::NIGHT_EXPOSURE,
            23.5..24.0 => {
                Self::NIGHT_EXPOSURE
                    - ((Self::NIGHT_EXPOSURE - Self::DAWN_EXPOSURE) / 6.0)
                        * (game_time as f32 - 18.0)
            }
            _ => unreachable!("game time is capped to 24h"),
        };

        if attributes.get_exposure_sensitivity() == exposure {
            return;
        }

        attributes.set_exposure_sensitivity(exposure);
    }
}
