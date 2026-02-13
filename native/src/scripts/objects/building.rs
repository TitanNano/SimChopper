mod fire;

use std::{any::Any, fmt::Debug};

use godot::builtin::Array;
use godot::classes::Texture2D;
use godot::classes::{MeshInstance3D, Node};
use godot::obj::{Gd, Inherits};
use godot_rust_script::godot_script_impl;
use godot_rust_script::{GodotScript, OnEditor, ScriptExportGroup, ScriptExportSubgroup};

use crate::util::Uf32;
use crate::{util::logger, world::city_data::TileCoords};

use fire::FireFeature;

trait BuildingFeature<N: Inherits<Node>>: Debug {
    fn process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
    fn physics_process(&mut self, _delta: f64, _node: &mut Gd<N>) {}
    fn dispatch_notification(&mut self, _notification: BuildingNotification) {}
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
        for item in &mut self.0 {
            item.process(delta, node);
        }
    }

    fn physics_process(&mut self, delta: f64, node: &mut Gd<N>) {
        for item in &mut self.0 {
            item.physics_process(delta, node);
        }
    }

    fn dispatch_notification(&mut self, notification: BuildingNotification) {
        for item in &mut self.0 {
            item.dispatch_notification(notification);
        }
    }
}

#[derive(Clone, Copy)]
enum BuildingNotification {
    WaterImpact(f64),
}

#[derive(ScriptExportGroup, Debug, Default)]
struct EventsConfig {
    #[export(flatten)]
    fire: Option<FireEventConfig>,
}

#[derive(ScriptExportSubgroup, Default, Debug)]
#[expect(clippy::struct_field_names)]
struct FireEventConfig {
    /// The positions of the emission points.
    emission_points: OnEditor<Gd<Texture2D>>,
    /// The normals of the emission points.
    emission_point_normals: OnEditor<Gd<Texture2D>>,
    /// The number of emission points inside the emission point texture.
    emission_point_count: Uf32,
}

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct Building {
    #[export(flatten)]
    pub events: EventsConfig,

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

        if let Some(config) = &self.events.fire {
            if let Some(ref mesh) = self.mesh {
                self.features
                    .push(Box::new(FireFeature::new(self.tile_coords, mesh, config)));
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
