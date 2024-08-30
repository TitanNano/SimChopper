use std::{collections::BTreeMap, ops::Not};

use anyhow::Context as _;
use derive_debug::Dbg;
use godot::builtin;
use godot::builtin::meta::ToGodot;
use godot::builtin::{Array, Dictionary};
use godot::engine::{Marker3D, Node, Node3D, Resource, Time};
use godot::obj::{Gd, NewAlloc};
use godot_rust_script::{
    godot_script_impl, CastToScript, GodotScript, RsRef, ScriptSignal, Signal,
};

use crate::objects::scene_object_registry;
use crate::util::async_support::{self, godot_task, GodotFuture, TaskHandle, ToSignalFuture};
use crate::util::logger;
use crate::world::city_coords_feature::CityCoordsFeature;
use crate::world::city_data::{self, TryFromDictionary};

#[derive(GodotScript, Dbg)]
#[script(base = Node)]
struct Buildings {
    #[dbg(skip)]
    pending_build_tasks: Vec<TaskHandle>,

    #[export]
    pub world_constants: Option<Gd<Resource>>,

    /// tile_coords, size, altitude
    #[signal]
    pub spawn_point_encountered: Signal<(Array<u32>, u8, u32)>,

    #[signal]
    pub loading_progress: Signal<u32>,

    base: Gd<Node>,
}

#[godot_script_impl]
impl Buildings {
    const TIME_BUDGET: u64 = 50;

    pub fn _process(&mut self, _delta: f64) {
        self.pending_build_tasks.retain(|task| task.is_pending());

        let tasks = self.pending_build_tasks.len();

        if tasks > 0 {
            logger::debug!(
                "World Buildings Node: {} active tasks!",
                self.pending_build_tasks.len()
            );
        }
    }

    fn world_constants(&self) -> &Gd<Resource> {
        self.world_constants
            .as_ref()
            .expect("world_constants should be set!")
    }

    pub fn build_async(&mut self, city: Dictionary) -> Gd<GodotFuture> {
        let world_constants = self.world_constants().clone();
        let mut base = self.base.clone();
        let mut slf: RsRef<Self> = base.to_script();
        let tree = base.get_tree().expect("Node must be part of the tree!");
        let (resolve, godot_future) = async_support::godot_future();

        let handle = godot_task(async move {
            let next_tick = builtin::Signal::from_object_signal(&tree, "process_frame");
            let time = Time::singleton();

            let city = match crate::world::city_data::City::try_from_dict(&city)
                .context("Failed to deserialize city data")
            {
                Ok(v) => v,
                Err(err) => {
                    logger::error!("{:?}", err);
                    return;
                }
            };

            let sea_level = city.simulator_settings.sea_level;
            let buildings = city.buildings;
            let tiles = city.tilelist;
            let city_coords_feature = CityCoordsFeature::new(world_constants, sea_level);

            logger::info!("starting to load buildings...");

            let mut count = 0;
            let mut start = time.get_ticks_msec();

            for building in buildings.into_values() {
                if (time.get_ticks_msec() - start) > Self::TIME_BUDGET {
                    slf.emit_progress(count);
                    count = 0;
                    start = time.get_ticks_msec();

                    let _: () = next_tick.to_future().await;
                }

                count += 1;

                if building.building_id == 0x00 {
                    logger::info!("skipping empty building");
                    continue;
                }

                Self::insert_building(&mut base, building, &tiles, &city_coords_feature);
            }

            slf.emit_progress(count);

            resolve(());
        });

        self.pending_build_tasks.push(handle);
        godot_future
    }

    /// Insert a new building into the world.
    fn insert_building(
        base: &mut Gd<Node>,
        building: city_data::Building,
        tiles: &BTreeMap<(u32, u32), city_data::Tile>,
        city_coords_feature: &CityCoordsFeature,
    ) {
        let building_size = building.size;
        let name = building.name.as_str();
        let building_id = building.building_id;
        let object = scene_object_registry::load_building(building_id);
        let tile_coords = building.tile_coords;
        let Some(tile) = tiles.get(&tile_coords) else {
            logger::error!("missing tile at {:?}", tile_coords);
            return;
        };

        let altitude: u32 = tile.altitude;

        let Some(object) = object else {
            logger::error!("unknown building \"{}\"", name);
            return;
        };

        if building_id == scene_object_registry::Buildings::Tarmac
            && is_spawn_point(&building, tiles)
        {
            logger::info!("encountered a spawn point: {:?}", building);
            let spawn_building = city_data::Building {
                building_id: scene_object_registry::Buildings::Hangar2 as u8,
                tile_coords,
                name: "Hangar".into(),
                size: 2,
            };

            Self::insert_building(base, spawn_building, tiles, city_coords_feature);
            CastToScript::<Buildings>::to_script(base).emit_spawn_point_encountered(
                Array::from(&[tile_coords.0, tile_coords.1]),
                2,
                altitude,
            );
        }

        let (Some(mut instance), instance_time) =
            with_timing(|| object.try_instantiate_as::<Node3D>())
        else {
            logger::error!("failed to instantiate building {}", name);
            return;
        };

        if !instance.get("tile_coords_array".into()).is_nil() {
            let mut array = Array::new();

            array.push(tile_coords.0);
            array.push(tile_coords.1);

            instance.set("tile_coords_array".into(), array.to_variant());
        }

        let mut location = city_coords_feature.get_building_coords(
            tile_coords.0,
            tile_coords.1,
            altitude,
            building_size,
        );

        // fix z fighting of flat buildings
        location.y += 0.1;

        let (_, insert_time) = with_timing(|| {
            Self::get_sector(base, tile_coords, city_coords_feature)
                .add_child_ex(instance.clone().upcast())
                .force_readable_name(true)
                .done();

            let Some(root) = base.get_tree().and_then(|tree| tree.get_current_scene()) else {
                logger::warn!("there is no active scene!");
                return;
            };

            instance.set_owner(root);
        });

        let (_, translate_time) =
            with_timing(|| instance.cast::<Node3D>().set_global_position(location));

        if instance_time > 100 {
            logger::warn!("\"{}\" is very slow to instantiate!", name);
        }

        if insert_time > 100 {
            logger::warn!("\"{}\" is very slow to insert!", name);
        }

        if translate_time > 100 {
            logger::warn!("\"{}\" is very slow to translate!", name);
        }
    }

    /// sector coordinates are expected to align with a step of 10
    fn get_sector(
        base: &mut <Self as GodotScript>::Base,
        tile_coords: (u32, u32),
        city_coords_feature: &CityCoordsFeature,
    ) -> Gd<Node3D> {
        const SECTOR_SIZE: u32 = 32;

        let sector_coords = (
            (tile_coords.0 / SECTOR_SIZE) * SECTOR_SIZE,
            (tile_coords.1 / SECTOR_SIZE) * SECTOR_SIZE,
        );

        let sector_name = {
            let (x, y) = sector_coords;

            format!("{}_{}", x, y)
        };

        base.get_node_or_null(sector_name.to_godot().into())
            .map(Gd::cast)
            .unwrap_or_else(|| {
                let mut sector: Gd<Node3D> = Marker3D::new_alloc().upcast();

                sector.set_name(sector_name.to_godot());

                base.add_child(sector.clone().upcast());

                sector.translate(city_coords_feature.get_world_coords(
                    sector_coords.0 + (SECTOR_SIZE / 2),
                    sector_coords.1 + (SECTOR_SIZE / 2),
                    0,
                ));

                if let Some(root) = base.get_tree().and_then(|tree| tree.get_current_scene()) {
                    sector.set_owner(root);
                };

                sector
            })
    }

    pub fn emit_spawn_point_encountered(&self, tile_coords: Array<u32>, size: u8, altitide: u32) {
        self.spawn_point_encountered
            .emit((tile_coords, size, altitide))
    }

    pub fn emit_progress(&self, new_building_count: u32) {
        self.loading_progress.emit(new_building_count);
    }
}

fn is_spawn_point(
    building: &city_data::Building,
    tiles: &BTreeMap<(u32, u32), city_data::Tile>,
) -> bool {
    let (x, y) = building.tile_coords;

    let x_miss = (x - 1..x + 3)
        .all(|index| {
            let Some(tile) = tiles.get(&(index, y)) else {
                logger::error!("unable to get tile at: x = {}, y = {}", index, y);
                return false;
            };

            let Some(building) = tile.building.as_ref() else {
                logger::warn!("tile has no building!");
                return false;
            };

            building.building_id == scene_object_registry::Buildings::Tarmac
        })
        .not();

    if x_miss {
        return false;
    }

    (y - 1..y + 3).all(|index| {
        let Some(tile) = tiles.get(&(x, index)) else {
            logger::error!("unable to get tile at: x = {}, y = {}", x, index);
            return false;
        };

        let Some(building) = tile.building.as_ref() else {
            return false;
        };

        building.building_id == scene_object_registry::Buildings::Tarmac
    })
}

fn with_timing<R>(cb: impl FnOnce() -> R) -> (R, u64) {
    let start = Time::singleton().get_ticks_msec();

    let result = cb();

    (result, Time::singleton().get_ticks_msec() - start)
}
