use godot::{
    builtin::{Array, NodePath, Transform3D, Vector3},
    engine::{MeshInstance3D, Node, Node3D, PackedScene, ResourceLoader, Time},
    meta::ToGodot,
    obj::{Gd, Inherits},
};
use godot_rust_script::{godot_script_impl, Context, GodotScript};
use rand::Rng;
use std::fmt::Debug;

use crate::{util::logger, world::city_data::TileCoords};

trait BuildingFeature<N: Inherits<Node>>: Debug {
    fn process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
    fn physics_process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
}

struct BuildingEventFlags(u8);

impl BuildingEventFlags {
    fn fire(&self) -> bool {
        self.0 & 0b00000001 == 1
    }
}

#[derive(Debug)]
struct Features<F: Debug + ?Sized>(Vec<Box<F>>);

impl<F: Debug + ?Sized> Features<F> {
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
}

const FIRE_SPAWNER_SCENE: &str = "res://resources/Objects/Spawner/fire_spawner.tscn";

#[derive(Debug)]
struct FireFeature {
    fire_scene: Option<Gd<Node3D>>,
    last_fire: u64,
    building_mesh: Gd<MeshInstance3D>,
    tile_coords: TileCoords,
}

impl FireFeature {
    fn new(tile_coords: TileCoords, mesh: &Gd<MeshInstance3D>) -> Self {
        Self {
            fire_scene: None,
            last_fire: 0,
            building_mesh: mesh.to_owned(),
            tile_coords,
        }
    }
}

impl<N: Inherits<Node>> BuildingFeature<N> for FireFeature {
    fn process(&mut self, _delta: f64, node: &mut Gd<N>) {
        let current_ticks = Time::singleton().get_ticks_msec();

        if let Some(ref mut scene) = self.fire_scene {
            if current_ticks - self.last_fire < 60_000 {
                return;
            }

            scene.queue_free();
            self.fire_scene = None;
            return;
        }

        let tick_delta = current_ticks - self.last_fire;
        let tick_boost = (tick_delta + 10_000) as f64 / 10_000.0;
        let rng = rand::thread_rng().gen_range(0.0..1.0);

        let chance = rng * tick_boost;

        if chance < 0.5 {
            return;
        }

        let Some(scene) = ResourceLoader::singleton()
            .load_ex(FIRE_SPAWNER_SCENE.into())
            .type_hint("PackedScene".into())
            .done()
        else {
            logger::error!("Failed to load fire_spawner scene: {}", FIRE_SPAWNER_SCENE);
            return;
        };

        let Some(mut scene_instance) = scene.cast::<PackedScene>().try_instantiate_as::<Node3D>()
        else {
            logger::error!(
                "Failed to instantiate fire_spawner scene as decendant of Node3D: {}",
                FIRE_SPAWNER_SCENE
            );
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

        self.fire_scene = Some(scene_instance);
        self.last_fire = current_ticks;

        logger::info!("Building started burning: {:?}", self.tile_coords);
    }
}

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct Building {
    #[export(flags = ["Fire:1"])]
    pub events: u8,

    #[export(node_path = ["MeshInstance3D"])]
    pub mesh_path: NodePath,

    pub tile_coords_array: Array<u32>,

    tile_coords: TileCoords,

    mesh: Option<Gd<MeshInstance3D>>,
    features: Features<dyn BuildingFeature<Node>>,

    base: Gd<Node>,
}

#[godot_script_impl]
impl Building {
    pub fn _ready(&mut self, mut context: Context) {
        let events = BuildingEventFlags(self.events);

        self.mesh = {
            let mesh_path = self.mesh_path.clone();
            let base = self.base.clone();

            context.reentrant_scope(self, || base.try_get_node_as(mesh_path.to_owned()))
        };

        self.tile_coords = (
            self.tile_coords_array.get(0).unwrap_or(0),
            self.tile_coords_array.get(1).unwrap_or(0),
        );

        if events.fire() {
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
}
