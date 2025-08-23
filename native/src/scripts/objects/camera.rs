use godot::builtin::math::FloatExt;
use godot::builtin::Signal;
use godot::classes::{Camera3D, CameraAttributesPhysical};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, RsRef};
use itertools::Itertools;

use crate::script_callable;
use crate::scripts::world::solar_setup::{ISolarSetup, SolarSetup};
use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Camera3D)]
struct Camera {
    #[export]
    pub solar_setup: OnEditor<RsRef<SolarSetup>>,

    base: Gd<Camera3D>,
}

#[derive(Debug)]
struct ExposureSetting {
    fstop: f32,
    shutter: f32,
    lux: f32,
}

#[godot_script_impl]
impl Camera {
    const NIGHT_SHUTTER: f32 = 2.0;
    const NIGHT_FSTOP: f32 = 1.4;
    const NIGHT_LUX: f32 = 2.2;

    const NIGHT2_SHUTTER: f32 = 2.0;
    const NIGHT2_FSTOP: f32 = 2.0;
    const NIGHT2_LUX: f32 = 20.0;

    const NIGHT3_SHUTTER: f32 = 4.0;
    const NIGHT3_FSTOP: f32 = 2.0;
    const NIGHT3_LUX: f32 = 40.0;

    const NIGHT4_SHUTTER: f32 = 8.0;
    const NIGHT4_FSTOP: f32 = 2.0;
    const NIGHT4_LUX: f32 = 160.0;

    const NIGHT5_SHUTTER: f32 = 15.0;
    const NIGHT5_FSTOP: f32 = 2.0;
    const NIGHT5_LUX: f32 = 160.0;

    const NIGHT6_SHUTTER: f32 = 30.0;
    const NIGHT6_FSTOP: f32 = 2.0;
    const NIGHT6_LUX: f32 = 320.0;

    const DAWN_SHUTTER: f32 = 60.0;
    const DAWN_FSTOP: f32 = 5.6;
    const DAWN_LUX: f32 = 2560.0;

    const MID_DAY_SHUTTER: f32 = 60.0;
    const MID_DAY_FSTOP: f32 = 22.0;
    const MID_DAY_LUX: f32 = 81920.0;

    const MAX_LIGHT_SHUTTER: f32 = 60.0;
    const MAX_LIGHT_FSTOP: f32 = 32.0;
    const MAX_LIGHT_LUX: f32 = 163840.0;

    const FSTOPS: [ExposureSetting; 9] = [
        ExposureSetting {
            fstop: Self::NIGHT_FSTOP,
            shutter: Self::NIGHT_SHUTTER,
            lux: Self::NIGHT_LUX,
        },
        ExposureSetting {
            fstop: Self::NIGHT2_FSTOP,
            shutter: Self::NIGHT2_SHUTTER,
            lux: Self::NIGHT2_LUX,
        },
        ExposureSetting {
            fstop: Self::NIGHT3_FSTOP,
            shutter: Self::NIGHT3_SHUTTER,
            lux: Self::NIGHT3_LUX,
        },
        ExposureSetting {
            fstop: Self::NIGHT4_FSTOP,
            shutter: Self::NIGHT4_SHUTTER,
            lux: Self::NIGHT4_LUX,
        },
        ExposureSetting {
            fstop: Self::NIGHT5_FSTOP,
            shutter: Self::NIGHT5_SHUTTER,
            lux: Self::NIGHT5_LUX,
        },
        ExposureSetting {
            fstop: Self::NIGHT6_FSTOP,
            shutter: Self::NIGHT6_SHUTTER,
            lux: Self::NIGHT6_LUX,
        },
        ExposureSetting {
            fstop: Self::DAWN_FSTOP,
            shutter: Self::DAWN_SHUTTER,
            lux: Self::DAWN_LUX,
        },
        ExposureSetting {
            fstop: Self::MID_DAY_FSTOP,
            shutter: Self::MID_DAY_SHUTTER,
            lux: Self::MID_DAY_LUX,
        },
        ExposureSetting {
            fstop: Self::MAX_LIGHT_FSTOP,
            shutter: Self::MAX_LIGHT_SHUTTER,
            lux: Self::MAX_LIGHT_LUX,
        },
    ];

    pub fn _ready(&mut self) {
        Signal::from_object_signal(&**self.solar_setup, "sky_brightness")
            .connect(&script_callable!(self, Self::_ready), 0);
    }

    pub fn _process(&mut self, _delta: f64) {
        let Some(mut attributes): Option<Gd<CameraAttributesPhysical>> = self
            .base
            .get_attributes()
            .and_then(|attr| attr.try_cast().ok())
        else {
            logger::warn!("Camera has no attributes!");
            return;
        };

        let brightness = self.solar_setup.environment_brightness();

        let (next_index, next) = Self::FSTOPS
            .iter()
            .find_position(|setting| setting.lux > brightness)
            .expect("there must always be a next");
        let closest = &Self::FSTOPS[next_index - 1];

        let diff = next.lux - closest.lux;

        let distance = (brightness - closest.lux).max(0.0) / diff;

        let fstop = closest.fstop.lerp(next.fstop, distance).snapped(0.1);
        let shutter = closest.shutter.lerp(next.shutter, distance).snapped(0.1);

        if attributes.get_aperture() == fstop && attributes.get_shutter_speed() == shutter {
            return;
        }

        attributes.set_aperture(fstop);
        attributes.set_shutter_speed(shutter);
    }
}
