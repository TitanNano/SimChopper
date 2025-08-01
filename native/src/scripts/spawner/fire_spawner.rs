use godot::builtin::math::ApproxEq;
use godot::builtin::{Transform3D, Vector3, Vector3Axis};
use godot::classes::{light_3d, FogVolume, Material, Node, OmniLight3D, ShaderMaterial};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node)]
pub struct FireSpawner {
    /// Node path to the fog volume for the flames.
    #[export]
    pub fire: OnEditor<Gd<FogVolume>>,

    /// Node path to the fog volume for the smoke.
    #[export]
    pub smoke: OnEditor<Gd<FogVolume>>,

    /// Node path to the omni light source of the fire.
    #[export]
    pub light_source: OnEditor<Gd<OmniLight3D>>,

    default_light_energy: f32,

    strength: f64,
}

#[godot_script_impl]
impl FireSpawner {
    pub fn _init(&mut self) {
        self.strength = 1.0;
    }

    pub fn _ready(&mut self) {
        logger::debug!("Init Fire spawner...");

        if let Some(material) = self.fire.get_material() {
            self.fire.set_material(
                &material
                    .duplicate()
                    .expect("Duplicating shouldn't fail")
                    .cast::<Material>(),
            );
        } else {
            logger::error!("Fire volume has no material assigned!");
        }

        self.default_light_energy = self.light_source.get_param(light_3d::Param::ENERGY);
    }

    pub fn resize(&mut self, size: Vector3) {
        let smoke_ratio = self.smoke.get_size() / self.fire.get_size();
        let fire_size = size * Vector3::new(1.0, 1.5, 1.0);
        let smoke_size = fire_size * smoke_ratio;
        let light_size = size;

        self.fire.set_size(fire_size);
        self.fire
            .set_transform(Transform3D::default().translated(Vector3::new(
                0.0,
                fire_size.y / 2.0 * 0.9,
                0.0,
            )));

        self.smoke.set_size(smoke_size);
        self.smoke
            .set_transform(Transform3D::default().translated(Vector3::new(
                0.0,
                smoke_size.y / 2.0 * 1.2,
                0.0,
            )));

        let light_max_size = match light_size.max_axis().unwrap_or(Vector3Axis::X) {
            Vector3Axis::X => light_size.x,
            Vector3Axis::Y => light_size.y,
            Vector3Axis::Z => light_size.z,
        };

        self.light_source
            .set_param(light_3d::Param::RANGE, light_size.length_squared() * 2.0);
        self.light_source
            .set_param(light_3d::Param::SIZE, light_max_size);
        self.light_source
            .set_transform(Transform3D::default().translated(Vector3::new(
                0.0,
                light_size.y / 2.0,
                0.0,
            )))
    }

    pub fn set_fire_strength(&mut self, strenght: f64) {
        self.strength = strenght;

        let mut material: Gd<ShaderMaterial> = self
            .fire
            .get_material()
            .expect("fire must have a material!")
            .cast();

        material.set_shader_parameter("strength", &strenght.to_variant());

        let light_source_energy: f32 = self.default_light_energy;

        self.light_source.set_param(
            light_3d::Param::ENERGY,
            light_source_energy * strenght as f32,
        );
    }

    pub fn is_dead(&self) -> bool {
        self.strength.approx_eq(&0.0)
    }
}
