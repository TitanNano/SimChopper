use godot::{
    builtin::{Dictionary, Transform3D, Vector3},
    engine::{IResource, Node3D, Resource},
    obj::{Base, Gd},
    register::{godot_api, GodotClass},
};

use crate::{
    road_navigation::RoadNavigation,
    util::logger,
    world::city_data::{Building, ToDictionary, TryFromDictionary},
};

use super::WorldConstants;

#[derive(GodotClass)]
#[class(base = Resource)]
pub(crate) struct RoadNavigationRes {
    inner: Option<RoadNavigation>,

    #[var(get = world_constants, set = set_world_constants)]
    #[export]
    world_constants: Option<Gd<WorldConstants>>,
}

#[godot_api]
impl RoadNavigationRes {
    #[func]
    fn world_constants(&self) -> Option<Gd<WorldConstants>> {
        self.world_constants.clone()
    }

    #[func]
    fn set_world_constants(&mut self, value: Gd<WorldConstants>) {
        self.inner = Some(RoadNavigation::new(value.clone()));
        self.world_constants = Some(value);
    }

    #[func]
    fn insert_node(&mut self, building: Dictionary, object: Gd<Node3D>) {
        let Some(inner) = self.inner.as_mut() else {
            logger::error!("Road Navigation resource is not ready!");
            return;
        };

        let building = match Building::try_from_dict(&building) {
            Ok(building) => building,
            Err(err) => {
                logger::error!("Failed to read building dict: {}", err);
                return;
            }
        };

        inner.insert_node(building, object);
    }

    #[func]
    fn get_nearest_node(&self, global_translation: Vector3) -> Dictionary {
        let Some(inner) = self.inner.as_ref() else {
            logger::error!("Road Navigation resource is not ready!");
            return Dictionary::new();
        };

        let node = inner.get_nearest_node(global_translation);

        let Some(building) = node.as_ref().map(|node| node.building()) else {
            logger::error!("Failed to get nearest navigation node!");
            return Dictionary::new();
        };

        building.to_dict()
    }

    #[func]
    fn get_next_node(
        &self,
        current: Dictionary,
        target: Dictionary,
        actor_orientation: Vector3,
    ) -> Dictionary {
        let current = match Building::try_from_dict(&current) {
            Ok(current) => current,
            Err(err) => {
                logger::error!("Invalid current building dictionary: {}", err);
                return Dictionary::new();
            }
        };

        let target = match Building::try_from_dict(&target) {
            Ok(target) => target,
            Err(err) => {
                logger::error!("Invalid target building dictionary: {}", err);
                return Dictionary::new();
            }
        };

        let Some(inner) = self.inner.as_ref() else {
            logger::error!("Road Navigation resource is not ready!");
            return Dictionary::new();
        };

        let Some(current_node) = inner.get_node(current.tile_coords) else {
            logger::error!("Current node does not exist: {:?}", current);
            return Dictionary::new();
        };

        let Some(target_node) = inner.get_node(target.tile_coords) else {
            logger::error!("Target node does not exist: {:?}", target);
            return Dictionary::new();
        };

        let dict = inner
            .get_next_node(&current_node, &target_node, actor_orientation)
            .building()
            .to_dict();

        dict
    }

    #[func]
    fn has_arrived(&self, location: Vector3, direction: Vector3, building: Dictionary) -> bool {
        let building = match Building::try_from_dict(&building) {
            Ok(building) => building,
            Err(err) => {
                logger::error!(
                    "Node is not a valid building dictionary: {}\n{:?}",
                    err,
                    building
                );
                return false;
            }
        };

        let Some(inner) = self.inner.as_ref() else {
            logger::error!("Road Navigation resource is not ready!");
            return false;
        };

        let Some(node) = inner.get_node(building.tile_coords) else {
            logger::error!("Navigation node does not exist!");
            return false;
        };

        match node.has_arrived(location, direction) {
            Ok(result) => result,
            Err(err) => {
                logger::error!("Failed perform arrival check: {:?}", err);
                false
            }
        }
    }

    #[func]
    fn get_random_node(&self) -> Dictionary {
        let Some(inner) = self.inner.as_ref() else {
            logger::error!("Road Navigation resource is not ready!");
            return Dictionary::new();
        };

        inner.get_random_node().building().to_dict()
    }

    #[func]
    fn get_global_transform(&self, node: Dictionary, direction: Vector3) -> Transform3D {
        let node = match Building::try_from_dict(&node) {
            Ok(node) => node,
            Err(err) => {
                logger::error!(
                    "Node is not a valid building dictionary: {}\n{:?}",
                    err,
                    node
                );
                return Transform3D::default();
            }
        };

        let Some(inner) = self.inner.as_ref() else {
            logger::error!("Road Navigation resource is not ready!");
            return Transform3D::default();
        };

        let Some(nav_node) = inner.get_node(node.tile_coords) else {
            logger::error!("Nav node does not exist: {:?}", node.tile_coords);
            return Transform3D::default();
        };

        nav_node.get_global_transform(direction)
    }
}

#[godot_api]
impl IResource for RoadNavigationRes {
    fn init(_base: Base<Resource>) -> Self {
        Self {
            inner: None,
            world_constants: None,
        }
    }
}
