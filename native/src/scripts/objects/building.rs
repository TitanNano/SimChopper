mod fire;

use godot::builtin::Array;
use godot::classes::{MeshInstance3D, Node};
use godot::global::PropertyHint;
use godot::meta::{FromGodot, GodotConvert, ToGodot};
use godot::obj::{Gd, Inherits};
use godot::prelude::ConvertError;
use godot_rust_script::{godot_script_impl, GodotScript, GodotScriptExport};
use std::{any::Any, fmt::Debug};

use crate::{util::logger, world::city_data::TileCoords};

use fire::FireFeature;

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
    type ToVia<'v> = Self::Via;

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

impl godot::prelude::Var for BuildingEventFlags {
    fn get_property(&self) -> Self::Via {
        self.to_godot()
    }

    fn set_property(&mut self, value: Self::Via) {
        *self = Self::from_godot(value);
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
