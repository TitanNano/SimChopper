use godot::{
    builtin::{math::ApproxEq, Array, Transform3D, Vector3},
    engine::{MeshInstance3D, Node, Node3D, PackedScene, Time},
    global::PropertyHint,
    meta::{FromGodot, GodotConvert, ToGodot},
    obj::{Gd, Inherits},
    prelude::ConvertError,
    tools::load,
};
use godot_rust_script::{godot_script_impl, CastToScript, GodotScript, GodotScriptExport, RsRef};
use rand::Rng;
use std::{any::Any, fmt::Debug};

use crate::{
    scripts::{FireSpawner, IFireSpawner},
    util::logger,
    world::city_data::TileCoords,
};

trait BuildingFeature<N: Inherits<Node>>: Debug {
    fn process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
    fn physics_process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
    fn dispatch_notification(&mut self, _notification: BuildingNotification) {}
}

#[derive(Debug, Default)]
struct BuildingEventFlags(u8);

impl BuildingEventFlags {
    fn fire(&self) -> bool {
        self.0 & 0b00000001 == 1
    }
}

impl GodotConvert for BuildingEventFlags {
    type Via = u8;
}

impl FromGodot for BuildingEventFlags {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Ok(Self(via))
    }
}

impl ToGodot for BuildingEventFlags {
    fn to_godot(&self) -> Self::Via {
        self.0
    }
}

impl GodotScriptExport for BuildingEventFlags {
    fn hint_string(_custom_hint: Option<PropertyHint>, custom_string: Option<String>) -> String {
        if let Some(custom_string) = custom_string {
            return custom_string;
        }

        String::from("Fire:1")
    }

    fn hint(custom: Option<PropertyHint>) -> PropertyHint {
        if let Some(custom) = custom {
            return custom;
        }

        PropertyHint::FLAGS
    }
}

#[derive(Debug)]
struct Features<F: Debug + ?Sized>(Vec<Box<F>>);

impl<F: Debug + Any + ?Sized> Features<F> {
    pub fn push(&mut self, feature: Box<F>) {
        self.0.push(feature);
    }
}

impl<F: Debug + ?Sized> Default for Features<F> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<N: Inherits<Node>> BuildingFeature<N> for Features<dyn BuildingFeature<N>> {
    fn process(&mut self, delta: f64, node: &mut Gd<N>) {
        for item in self.0.iter_mut() {
            item.process(delta, node);
        }
    }

    fn physics_process(&mut self, delta: f64, node: &mut Gd<N>) {
        for item in self.0.iter_mut() {
            item.physics_process(delta, node);
        }
    }

    fn dispatch_notification(&mut self, notification: BuildingNotification) {
        for item in self.0.iter_mut() {
            item.dispatch_notification(notification);
        }
    }
}

#[derive(Clone, Copy)]
enum BuildingNotification {
    WaterImpact(f64),
}

#[derive(Debug)]
struct FireFeature {
    packed_fire_scene: Gd<PackedScene>,
    fire_scene: Option<RsRef<FireSpawner>>,
    last_fire: u64,
    fire_strength: f64,
    last_fire_strength: f64,
    building_mesh: Gd<MeshInstance3D>,
    tile_coords: TileCoords,
}

impl FireFeature {
    const FIRE_SPAWNER_SCENE: &'static str = "res://resources/Objects/Spawner/fire_spawner.tscn";
    const RECOVERY_RATE: f64 = 0.01;
    const WATER_IMPACT_RATE: f64 = 0.2;

    fn new(tile_coords: TileCoords, mesh: &Gd<MeshInstance3D>) -> Self {
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

    fn is_dead(&self) -> bool {
        self.fire_strength.approx_eq(&0.0)
    }

    fn is_recovering(&self) -> bool {
        !self.is_dead() && self.fire_strength - self.last_fire_strength >= 0.0
    }

    fn update_fire_strength(&mut self, fire: &mut RsRef<FireSpawner>) {
        if self.fire_strength == self.last_fire_strength {
            return;
        }

        fire.set_fire_strength(self.fire_strength);
        self.last_fire_strength = self.fire_strength;
    }

    fn recover_fire_strength(&mut self, delta: f64) {
        if !self.is_recovering() {
            return;
        }

        self.fire_strength = (self.fire_strength + Self::RECOVERY_RATE * delta).min(1.0);
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
        let tick_damp = (tick_delta as f64 / 10_000.0).min(1.0);
        let rng = rand::thread_rng().sample::<f64, _>(rand::distributions::OpenClosed01);

        let chance = rng * tick_damp;

        if chance < 0.5 {
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

        scene_instance.call_deferred("resize".into(), &[aabb_size.to_variant()]);

        node.upcast_mut()
            .add_child_ex(scene_instance.clone().upcast())
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
                self.fire_strength =
                    (self.fire_strength - Self::WATER_IMPACT_RATE * delta).max(0.0);
            }
        }
    }
}

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct Building {
    #[export]
    pub events: BuildingEventFlags,

    #[export]
    pub mesh: Option<Gd<MeshInstance3D>>,

    pub tile_coords_array: Array<u32>,

    tile_coords: TileCoords,

    features: Features<dyn BuildingFeature<Node>>,

    base: Gd<Node>,
}

#[godot_script_impl]
impl Building {
    pub fn _ready(&mut self) {
        self.tile_coords = (
            self.tile_coords_array.get(0).unwrap_or(0),
            self.tile_coords_array.get(1).unwrap_or(0),
        );

        if self.events.fire() {
            if let Some(ref mesh) = self.mesh {
                self.features
                    .push(Box::new(FireFeature::new(self.tile_coords, mesh)));
            } else {
                logger::warn!("Unable to instantiate FireFeature because no mesh has been set.");
            }
        }
    }

    pub fn _process(&mut self, delta: f64) {
        self.features.process(delta, &mut self.base);
    }

    pub fn _physics_process(&mut self, delta: f64) {
        self.features.physics_process(delta, &mut self.base);
    }

    pub fn impact_water(&mut self, delta: f64) {
        let notification = BuildingNotification::WaterImpact(delta);

        self.dispatch_notification(notification);
    }

    fn dispatch_notification(&mut self, notification: BuildingNotification) {
        self.features.dispatch_notification(notification);
    }
}
