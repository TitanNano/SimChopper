use godot::builtin::math::ApproxEq;
use godot::builtin::{NodePath, Transform3D, Vector3, Vector3Axis};
use godot::classes::{light_3d, FogVolume, Node, OmniLight3D, ShaderMaterial};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node)]
pub struct FireSpawner {
    /// Node path to the fog volume for the flames.
    #[export(node_path = ["FogVolume"])]
    pub fire_path: NodePath,
    fire: Option<Gd<FogVolume>>,

    /// Node path to the fog volume for the smoke.
    #[export(node_path = ["FogVolume"])]
    pub smoke_path: NodePath,
    smoke: Option<Gd<FogVolume>>,

    /// Node path to the omni light source of the fire.
    #[export(node_path = ["OmniLight3D"])]
    pub light_source_path: NodePath,
    light_source: Option<Gd<OmniLight3D>>,

    default_light_energy: f32,

    strength: f64,

    base: Gd<Node>,
}

#[godot_script_impl]
impl FireSpawner {
    pub fn _init(&mut self) {
        self.strength = 1.0;
    }

    pub fn _ready(&mut self) {
        logger::debug!("Init Fire spawner...");
        self.fire = self.base.try_get_node_as(self.fire_path.clone());
        self.smoke = self.base.try_get_node_as(self.smoke_path.clone());
        self.light_source = self.base.try_get_node_as(self.light_source_path.clone());

        if let Some(ref mut fire) = self.fire {
            if let Some(material) = fire.get_material() {
                fire.set_material(
                    material
                        .duplicate()
                        .expect("Duplicating shouldn't fail")
                        .cast(),
                );
            } else {
                logger::error!("Fire volume has no material assigned!");
            }
        } else {
            logger::error!("No fire volume has been setup!");
        }

        if let Some(ref mut light_source) = self.light_source {
            self.default_light_energy = light_source.get_param(light_3d::Param::ENERGY);
        }
    }

    pub fn resize(&mut self, size: Vector3) {
        let Some(ref mut fire) = self.fire else {
            logger::error!("Failed to resize fire spawner! No fire setup!");
            return;
        };

        let Some(ref mut smoke) = self.smoke else {
            logger::error!("Failed to resize fire spawner! No smoke setup!");
            return;
        };

        let Some(ref mut light_source) = self.light_source else {
            logger::error!("Failed to resize fire spawner! No light source setup!");
            return;
        };

        let smoke_ratio = smoke.get_size() / fire.get_size();
        let fire_size = size * Vector3::new(1.0, 1.5, 1.0);
        let smoke_size = fire_size * smoke_ratio;
        let light_size = size;

        fire.set_size(fire_size);
        fire.set_transform(Transform3D::default().translated(Vector3::new(
            0.0,
            fire_size.y / 2.0 * 0.9,
            0.0,
        )));

        smoke.set_size(smoke_size);
        smoke.set_transform(Transform3D::default().translated(Vector3::new(
            0.0,
            smoke_size.y / 2.0 * 1.2,
            0.0,
        )));

        let light_max_size = match light_size.max_axis().unwrap_or(Vector3Axis::X) {
            Vector3Axis::X => light_size.x,
            Vector3Axis::Y => light_size.y,
            Vector3Axis::Z => light_size.z,
        };

        light_source.set_param(light_3d::Param::RANGE, light_size.length_squared());
        light_source.set_param(light_3d::Param::SIZE, light_max_size);
        light_source.set_transform(Transform3D::default().translated(Vector3::new(
            0.0,
            light_size.y / 2.0,
            0.0,
        )))
    }

    pub fn set_fire_strength(&mut self, strenght: f64) {
        let Some(ref fire) = self.fire else {
            return;
        };

        let Some(ref mut light_source) = self.light_source else {
            return;
        };

        self.strength = strenght;

        let mut material: Gd<ShaderMaterial> = fire
            .get_material()
            .expect("fire must have a material!")
            .cast();

        material.set_shader_parameter("strength".into(), strenght.to_variant());

        let light_source_energy: f32 = self.default_light_energy;

        light_source.set_param(
            light_3d::Param::ENERGY,
            light_source_energy * strenght as f32,
        );
    }

    pub fn is_dead(&self) -> bool {
        self.strength.approx_eq(&0.0)
    }
}
