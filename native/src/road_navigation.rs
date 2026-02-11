use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use godot::builtin::math::ApproxEq;
use godot::builtin::{Transform3D, VarDictionary, Vector3};
use godot::classes::Node3D;
use godot::global::snappedf;
use godot::obj::{Gd, OnEditor};
use godot::prelude::{godot_api, GodotClass};
use num::ToPrimitive;
use rand::distr::Uniform;
use rand::Rng;

use crate::world::city_coords_feature::CityCoordsFeature;
use crate::world::city_data::TryFromDictionary;
use crate::{
    resources::WorldConstants,
    world::city_data::{Building, TileCoords},
};

enum Corners {
    BottomRight,
    BottomLeft,
    TopLeft,
    TopRight,
}

impl Corners {
    fn direction(&self) -> Vector3 {
        match self {
            Self::TopLeft => Vector3::FORWARD + Vector3::LEFT,
            Self::TopRight => Vector3::FORWARD + Vector3::RIGHT,
            Self::BottomLeft => Vector3::BACK + Vector3::LEFT,
            Self::BottomRight => Vector3::BACK + Vector3::RIGHT,
        }
    }

    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x24 => Some(Self::BottomRight),
            0x25 => Some(Self::BottomLeft),
            0x26 => Some(Self::TopLeft),
            0x23 => Some(Self::TopRight),
            _ => None,
        }
    }
}

enum Direction {
    Forward,
    Back,
    Left,
    Right,
}

impl Direction {
    // Direction degrees in radiants.
    const FORWARD: f64 = 0.0;
    const BACK: f64 = 180.0f64.to_radians();
    const BACK_NEGATIVE: f64 = -180.0f64.to_radians();
    const LEFT: f64 = 90.0f64.to_radians();
    const RIGHT: f64 = -90.0f64.to_radians();
}

impl TryFrom<f64> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value {
            Self::FORWARD => Ok(Self::Forward),
            Self::BACK | Self::BACK_NEGATIVE => Ok(Self::Back),
            Self::LEFT => Ok(Self::Left),
            Self::RIGHT => Ok(Self::Right),
            _ => Err(anyhow!("Invalid direction angle: {}", value)),
        }
    }
}

struct NavNode {
    building: Building,
    object: Gd<Node3D>,
    neighbors: OnceLock<Box<[(u32, u32)]>>,
}

#[derive(Clone)]
pub(crate) struct NavNodeRef<'n> {
    node: &'n NavNode,
    world_constants: &'n Gd<WorldConstants>,
}

impl<'n> NavNodeRef<'n> {
    fn new(node: &'n NavNode, world_constants: &'n Gd<WorldConstants>) -> Self {
        Self {
            node,
            world_constants,
        }
    }

    pub fn get_global_transform(&self, direction: Vector3) -> Transform3D {
        const RAD_90_DEG: f64 = 90.0f64.to_radians();
        const TILE_QUARTER: f32 = 4.0;
        const TILE_EIGTH: f32 = 8.0;

        let transform = self.node.object.get_global_transform();
        let building_id = self.node.building.id;

        let width = self.world_constants.bind().tile_size();
        let raw_angle = Vector3::FORWARD.signed_angle_to(direction, Vector3::UP);
        let angle = snappedf(f64::from(raw_angle), RAD_90_DEG);

        let offset = match (Corners::from_u8(building_id), direction) {
            (None, _) => {
                (f32::from(width) / 4.0)
                    * Vector3::RIGHT.rotated(
                        Vector3::UP,
                        angle
                            .to_f32()
                            .expect("should be ok to cast from f64 to f32"),
                    )
            }

            (Some(corner), Vector3::ZERO) => {
                let dir = corner.direction();
                let offset = (Vector3::RIGHT + Vector3::BACK) * (f32::from(width) / TILE_QUARTER);

                dir * offset
            }

            (Some(corner), _) => {
                let dir = corner.direction();

                let offset = (Vector3::BACK + Vector3::RIGHT) * (f32::from(width) / TILE_EIGTH);

                let multiplier = match (
                    Direction::try_from(angle).expect("angle should be properly snapped!"),
                    corner,
                ) {
                    (Direction::Forward | Direction::Left, Corners::BottomLeft)
                    | (Direction::Back | Direction::Left, Corners::BottomRight)
                    | (Direction::Forward | Direction::Right, Corners::TopLeft)
                    | (Direction::Back | Direction::Right, Corners::TopRight) => 1.0,

                    (Direction::Back | Direction::Right, Corners::BottomLeft)
                    | (Direction::Forward | Direction::Right, Corners::BottomRight)
                    | (Direction::Back | Direction::Left, Corners::TopLeft)
                    | (Direction::Forward | Direction::Left, Corners::TopRight) => 2.0,
                };

                dir * offset * multiplier
            }
        };

        transform.translated(offset)
    }

    pub fn tile_coords(&self) -> TileCoords {
        self.node.building.tile_coords
    }

    pub fn has_arrived(&self, location: Vector3, direction: Vector3) -> bool {
        let target = self.get_global_transform(direction).origin;

        // remove Y from all comparisons
        let target = Vector3::new(target.x, 0.0, target.z);
        let location = Vector3::new(location.x, 0.0, location.z);

        let distance = location.distance_to(target);

        distance < 4.0
    }
}

pub(crate) struct RoadNavigation {
    network: BTreeMap<TileCoords, NavNode>,
    world_constants: Gd<WorldConstants>,
    rand_distribution: Uniform<usize>,
    city_coords_feature: CityCoordsFeature,
}

impl RoadNavigation {
    pub fn new(world_constants: Gd<WorldConstants>) -> Self {
        Self {
            network: BTreeMap::default(),
            world_constants: world_constants.clone(),
            rand_distribution: Uniform::new(0, 1).expect("we have constant bounds"),
            city_coords_feature: CityCoordsFeature::new(world_constants, 0),
        }
    }

    pub fn insert_node(&mut self, node: Building, object: Gd<Node3D>) {
        let tile_coords = node.tile_coords;
        let node = NavNode {
            building: node,
            object,
            neighbors: OnceLock::new(),
        };

        self.network.insert(tile_coords, node);
        self.rand_distribution =
            Uniform::new(0, self.network.len()).expect("lower bound is always zero");
    }

    #[inline]
    pub fn try_node(&self, coords: TileCoords) -> Option<NavNodeRef<'_>> {
        let node = self.network.get(&coords)?;

        Some(NavNodeRef::new(node, &self.world_constants))
    }

    #[inline]
    pub fn node(&self, coords: TileCoords) -> NavNodeRef<'_> {
        self.try_node(coords)
            .expect("we are absolutely sure the node")
    }

    pub fn get_neighbors(&self, tile_coords: TileCoords) -> Option<&[TileCoords]> {
        let cache = &self.network.get(&tile_coords)?.neighbors;

        let neighbors = cache.get_or_init(|| {
            let (x, y) = tile_coords;

            let neighbors = [
                y.checked_sub(1).map(|y| (x, y)),
                x.checked_sub(1).map(|x| (x, y)),
                Some((x + 1, y)),
                Some((x, y + 1)),
            ]
            .into_iter()
            .flatten()
            .filter_map(|tile_coords| self.network.get(&tile_coords))
            .map(|node| node.building.tile_coords)
            .collect();

            neighbors
        });

        Some(neighbors)
    }

    pub fn get_nearest_node(&self, global_translation: Vector3) -> Option<NavNodeRef<'_>> {
        let own_coords = {
            let (x, y, _altitude) = self
                .city_coords_feature
                .tile_coordinates(global_translation);

            (x, y)
        };

        // fas track, pick node at global transform
        if let Some(node) = self.try_node(own_coords) {
            return Some(node);
        }

        let (mut low, mut low_node) = self.network.first_key_value()?;
        let (mut high, mut high_node) = self.network.last_key_value()?;

        loop {
            let distance_low = {
                let v = global_translation - low_node.object.get_global_position();

                Vector3::new(v.x, 0.0, v.z)
            };

            let distance_high = {
                let v = high_node.object.get_global_position() - global_translation;

                Vector3::new(v.x, 0.0, v.z)
            };

            if distance_low.is_zero_approx() {
                break self.try_node(*low);
            }

            if distance_high.is_zero_approx() {
                break self.try_node(*high);
            }

            let (new_low, new_low_node) = {
                let ordering = low.cmp(&own_coords);

                let range = match ordering {
                    Ordering::Less | Ordering::Equal => (*low)..=own_coords,
                    Ordering::Greater => own_coords..=*low,
                };

                let mut node_range = self.network.range(range);

                let maybe_node = match ordering {
                    Ordering::Less => node_range.next_back(),
                    Ordering::Equal | Ordering::Greater => node_range.next(),
                };

                let Some((new, node)) = maybe_node else {
                    break self.try_node(*high);
                };

                (new, node)
            };

            let (new_high, new_high_node) = {
                let ordering = own_coords.cmp(high);

                let range = match ordering {
                    Ordering::Less | Ordering::Equal => own_coords..=*high,
                    Ordering::Greater => *high..=own_coords,
                };

                let mut node_range = self.network.range(range);

                let maybe_node = match ordering {
                    Ordering::Less => node_range.next(),
                    Ordering::Equal | Ordering::Greater => node_range.next_back(),
                };

                let Some((new, node)) = maybe_node else {
                    break self.try_node(*high);
                };

                (new, node)
            };

            if new_low > new_high {
                high = new_low;
                high_node = new_low_node;
                low = new_high;
                low_node = new_low_node;
                continue;
            }

            if new_low == new_high {
                break self.try_node(*new_low);
            }

            if low == new_low && high == new_high {
                match distance_low
                    .length_squared()
                    .total_cmp(&distance_high.length_squared())
                {
                    Ordering::Less | Ordering::Equal => {
                        break self.try_node(*low);
                    }

                    Ordering::Greater => {
                        break self.try_node(*high);
                    }
                }
            }

            low = new_low;
            low_node = new_low_node;
            high = new_high;
            high_node = new_high_node;
        }
    }

    pub fn get_next_node<'n>(
        &'n self,
        current: &NavNodeRef<'n>,
        target: &NavNodeRef<'n>,
        actor_orientation: Vector3,
    ) -> NavNodeRef<'n> {
        let current_location = current.get_global_transform(Vector3::ZERO).origin;
        let target_location = target.get_global_transform(Vector3::ZERO).origin;

        // current and target might be the same if we arrived at the target node.
        if current_location
            .distance_squared_to(target_location)
            .approx_eq(&0.0)
        {
            return target.clone();
        }

        let dir_target =
            current_location.direction_to(target.get_global_transform(Vector3::ZERO).origin);

        let neighbors = self
            .get_neighbors(current.tile_coords())
            .expect("curent node must exist");

        let (_, next) =
            neighbors
                .iter()
                .fold((360.0, current.to_owned()), |(closest, next), coords| {
                    let Some(neighbor) = self.try_node(*coords) else {
                        return (closest, next);
                    };

                    let neighbor_location = neighbor.get_global_transform(Vector3::ZERO).origin;
                    let dir = current_location.direction_to(neighbor_location);
                    let angle_actor_orientation = dir.angle_to(actor_orientation);
                    let angle = dir.angle_to(dir_target);

                    // multiplying the angle between the target and the neighbor with the
                    // angle between the current actor orientation and the required actor
                    // orientation, adds bias towards a neighbor that is in the direction
                    // of the actors current orientation.
                    let weight = angle * (angle_actor_orientation / 2.0);

                    if closest < weight {
                        return (closest, next);
                    }

                    (weight, neighbor)
                });

        next
    }

    pub fn get_random_node(&self) -> NavNodeRef<'_> {
        let index = rand::rng().sample(self.rand_distribution);

        let node = self
            .network
            .values()
            .nth(index)
            .expect("index must be in range");

        NavNodeRef {
            node,
            world_constants: &self.world_constants,
        }
    }
}

/// Configuration resource to setup road navigation for vehicles.
#[derive(GodotClass)]
#[class(base = Resource, init)]
pub struct RoadNavigationConfig {
    /// The world constants to be used by the road navigation.
    #[export]
    world_constants: OnEditor<Gd<WorldConstants>>,

    instance: OnceCell<RoadNavigation>,
}

impl RoadNavigationConfig {
    pub(crate) fn road_navigation(&self) -> &RoadNavigation {
        self.instance
            .get_or_init(|| RoadNavigation::new(self.world_constants.clone()))
    }

    pub(crate) fn road_navigation_mut(&mut self) -> &mut RoadNavigation {
        // make sure instance is initialized.
        self.road_navigation();

        self.instance.get_mut().expect("we just initialized")
    }
}

#[godot_api]
impl RoadNavigationConfig {
    /// Insert a node into the road navigation graph.
    #[func]
    #[expect(clippy::needless_pass_by_value)]
    pub fn insert_node(&mut self, building: VarDictionary, scene_node: Gd<Node3D>) {
        let building = Building::try_from_dict(&building)
            .expect("building dictionary must be a vaild SC2K building.");

        self.road_navigation_mut().insert_node(building, scene_node);
    }
}
