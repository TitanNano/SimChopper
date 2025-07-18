use godot::builtin::math::FloatExt;
use godot::obj::Gd;
use godot::{builtin::Signal, classes::Camera3D};
use godot_rust_script::{godot_script_impl, GodotScript, RsRef};

use crate::script_callable;
use crate::{
    scripts::world::solar_setup::{ISolarSetup, SolarSetup},
    util::logger,
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

    pub fn _ready(&mut self) {
        let Some(ref solar_setup) = self.solar_setup else {
            logger::error!("missing solar setup in Camera!");
            return;
        };

        Signal::from_object_signal(solar_setup, "sky_brightness")
            .connect(&script_callable!(self, Self::_ready), 0);
    }

    pub fn _process(&mut self, _delta: f64) {
        let Some(solar_setup) = self.solar_setup.as_mut() else {
            warn!("Solar setup is not available!");
            return;
        };

        let Some(mut attributes) = self.base.get_attributes() else {
            warn!("Camera has no attributes!");
            return;
        };

        let sky_brightness = solar_setup.sky_brightness();
        let sun_zenite_distance = solar_setup.sun_zenite_distance();

        let day_exposure = Self::MID_DAY_EXPOSURE.lerp(Self::DAWN_EXPOSURE, sun_zenite_distance);

        let night_exposure = interpolate(
            Self::DAWN_EXPOSURE,
            Self::NIGHT_EXPOSURE,
            1.0 - sky_brightness,
        );

        let exposure = if sun_zenite_distance < 1.0 {
            day_exposure
        } else {
            night_exposure
        }
        .floor();

        if attributes.get_exposure_sensitivity() == exposure {
            return;
        }

        attributes.set_exposure_sensitivity(exposure);

        logger::debug!(
            "Updating camera exposure: {}",
            attributes.get_exposure_sensitivity()
        );
        logger::debug!("sky brightness: {}", sky_brightness);
    }
}

fn interpolate(from: f32, to: f32, weight: f32) -> f32 {
    let range = to - from;

    from + (weight.powf(4.0) * range)
}

#[cfg(test)]
mod test {
    use godot::builtin::math::ApproxEq;

    use super::interpolate;

    #[test]
    fn qerp_start() {
        let value = interpolate(4.5, 7.3, 0.0);

        assert_eq!(value, 4.5);
    }

    #[test]
    fn qerp_end() {
        let value = interpolate(4.5, 7.3, 1.0);

        assert_eq!(value, 7.3);
    }

    #[test]
    fn qerp_sample() {
        let value = interpolate(0.0, 1.0, 0.7);

        assert!(value.approx_eq(&0.49));
    }
}
