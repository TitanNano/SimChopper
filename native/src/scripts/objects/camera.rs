use godot::{builtin::Dictionary, classes::Camera3D, engine::Node3D, obj::Gd};
use godot_rust_script::{godot_script_impl, CastToScript, GodotScript, RsRef};
use itertools::Itertools;

use crate::{
    info,
    scripts::world::solar_setup::{ISolarSetup, SolarSetup},
    warn,
};

#[derive(GodotScript, Debug)]
#[script(base = Camera3D)]
struct Camera {
    #[export]
    pub exposure_times: Dictionary,

    #[export]
    pub solar_setup_node: Option<Gd<Node3D>>,

    solar_setup: Option<RsRef<SolarSetup>>,

    base: Gd<Camera3D>,
}

#[godot_script_impl]
impl Camera {
    pub fn _ready(&mut self) {
        self.solar_setup = self.solar_setup_node.clone().map(|node| node.into_script());
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

        let game_time = solar_setup.get_ingame_time_h();

        let exposure = self
            .exposure_times
            .iter_shared()
            .sorted_by(|(key_a, _), (key_b, _)| key_a.to::<f64>().total_cmp(&key_b.to::<f64>()))
            .rev()
            .find(|(key, _)| key.to::<f64>() <= game_time)
            .map(|(_, value)| value.to::<f32>())
            .unwrap_or(0.0);

        if attributes.get_exposure_sensitivity() == exposure {
            return;
        }

        info!("Updating camera exposure to {} at {}", exposure, game_time);

        attributes.set_exposure_sensitivity(exposure);
    }
}
