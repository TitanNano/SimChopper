use godot::builtin::{math::ApproxEq, Transform3D, Vector3};
use godot::classes::{MeshInstance3D, Node, Node3D, PackedScene, Time};
use godot::meta::ToGodot;
use godot::obj::{Gd, Inherits};
use godot::tools::load;
use godot_rust_script::{CastToScript, RsRef};
use num::ToPrimitive;
use rand::Rng;

use crate::scripts::{FireSpawner, IFireSpawner};
use crate::util::logger;
use crate::world::city_data::TileCoords;

use super::{BuildingFeature, BuildingNotification};

#[derive(Debug)]
pub(super) struct FireFeature {
    packed_fire_scene: Gd<PackedScene>,
    fire_scene: Option<RsRef<FireSpawner>>,
    last_fire: u64,
    fire_strength: f32,
    last_fire_strength: f32,
    building_mesh: Gd<MeshInstance3D>,
    tile_coords: TileCoords,
}

impl FireFeature {
    const FIRE_SPAWNER_SCENE: &'static str = "res://resources/Objects/Spawner/fire_spawner.tscn";
    const RECOVERY_RATE: f32 = 0.01;
    const WATER_IMPACT_RATE: f32 = 0.2;

    pub fn new(tile_coords: TileCoords, mesh: &Gd<MeshInstance3D>) -> Self {
        let packed = load(Self::FIRE_SPAWNER_SCENE);

        Self {
            packed_fire_scene: packed,
            fire_scene: None,
            fire_strength: 1.0,
            last_fire_strength: 1.0,
            last_fire: 0,
            building_mesh: mesh.to_owned(),
            tile_coords,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.fire_strength.approx_eq(&0.0)
    }

    pub fn is_recovering(&self) -> bool {
        !self.is_dead() && self.fire_strength - self.last_fire_strength >= 0.0
    }

    pub fn update_fire_strength(&mut self, fire: &mut RsRef<FireSpawner>) {
        if self.fire_strength.approx_eq(&self.last_fire_strength) {
            return;
        }

        fire.set_fire_strength(self.fire_strength);
        self.last_fire_strength = self.fire_strength;
    }

    pub fn recover_fire_strength(&mut self, delta: f64) {
        if !self.is_recovering() {
            return;
        }

        self.fire_strength = (self.fire_strength
            + Self::RECOVERY_RATE * delta.to_f32().expect("delta can be truncated"))
        .min(1.0);
    }
}

impl<N: Inherits<Node>> BuildingFeature<N> for FireFeature {
    fn process(&mut self, _delta: f64, node: &mut Gd<N>) {
        let current_ticks = Time::singleton().get_ticks_msec();

        if let Some(mut scene) = self.fire_scene.clone() {
            self.update_fire_strength(&mut scene);

            if !self.is_dead() {
                self.last_fire = current_ticks;
                return;
            }

            if current_ticks - self.last_fire < 60_000 {
                return;
            }

            scene.queue_free();
            self.fire_scene = None;
            self.last_fire = current_ticks;
            self.fire_strength = 1.0;
            self.last_fire_strength = 1.0;
            return;
        }

        let tick_delta = current_ticks - self.last_fire;
        let tick_damp = (tick_delta
            .to_f64()
            .expect("tick delta is expected to fit in f64")
            / 10_000.0)
            .min(1.0);
        let rng = rand::rng().sample::<f64, _>(rand::distr::OpenClosed01);

        let chance = rng * tick_damp;

        if chance < 0.9 {
            return;
        }

        logger::debug!("Building will burn! (tick_delta: {tick_delta}, tick_boost: {tick_damp}, rng: {rng}, chance: {chance})");

        let Some(mut scene_instance) = self.packed_fire_scene.try_instantiate_as::<Node3D>() else {
            logger::error!("Failed to instantiate fire_spawner scene as decendant of Node3D");
            return;
        };

        let aabb = self.building_mesh.get_aabb();
        let aabb_size = (Transform3D::new(self.building_mesh.get_basis(), Vector3::default())
            * aabb.size)
            .abs();

        scene_instance.call_deferred("resize", &[aabb_size.to_variant()]);

        node.upcast_mut()
            .add_child_ex(&scene_instance)
            .force_readable_name(true)
            .done();

        self.fire_scene = Some(scene_instance.to_script());

        logger::info!("Building started burning: {:?}", self.tile_coords);
    }

    fn physics_process(&mut self, delta: f64, _node: &mut Gd<N>) {
        self.recover_fire_strength(delta);
    }

    fn dispatch_notification(&mut self, notification: BuildingNotification) {
        match notification {
            BuildingNotification::WaterImpact(delta) => {
                self.fire_strength = (self.fire_strength
                    - Self::WATER_IMPACT_RATE * delta.to_f32().expect("delta can be truncated"))
                .max(0.0);
            }
        }
    }
}
