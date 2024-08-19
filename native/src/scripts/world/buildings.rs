use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Debug,
    ops::Not,
};

use anyhow::Context;
use derive_debug::Dbg;
use godot::{
    builtin::{meta::ToGodot, Array, Dictionary},
    engine::{Marker3D, Node, Node3D, Resource, Time},
    obj::{Gd, NewAlloc},
};
use godot_rust_script::{godot_script_impl, GodotScript, ScriptSignal, Signal};

use crate::{
    objects::scene_object_registry,
    util::logger,
    world::{
        city_coords_feature::CityCoordsFeature,
        city_data::{self, TryFromDictionary},
    },
};

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct Buildings {
    #[export]
    pub world_constants: Option<Gd<Resource>>,

    city_coords_feature: CityCoordsFeature,
    job_runner: Option<LocalJobRunner<city_data::Building, Self>>,

    /// tile_coords, size, altitude
    #[signal]
    spawn_point_encountered: Signal<(Array<u32>, u8, u32)>,

    #[signal]
    loading_progress: Signal<u32>,

    #[signal]
    ready: Signal<()>,

    base: Gd<Node>,
}

#[godot_script_impl]
impl Buildings {
    pub fn _process(&mut self, _delta: f64) {
        if let Some(mut job_runner) = self.job_runner.take() {
            let progress = job_runner.poll(self);

            self.job_runner = Some(job_runner);

            match progress {
                0 => self.ready.emit(()),
                progress => self.loading_progress.emit(progress),
            }
        }
    }

    fn world_constants(&self) -> &Gd<Resource> {
        self.world_constants
            .as_ref()
            .expect("world_constants should be set!")
    }

    pub fn build_async(&mut self, city: Dictionary) {
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

        self.city_coords_feature =
            CityCoordsFeature::new(self.world_constants().to_owned(), sea_level);

        logger::info!("starting to load buildings...");

        let mut job_runner = LocalJobRunner::new(
            move |host: &mut Self, building: city_data::Building| {
                if building.building_id == 0x00 {
                    logger::info!("skipping empty building");
                    return;
                }

                host.insert_building(building, &tiles);
            },
            50,
        );

        let buildings_array = buildings.into_values().collect();

        job_runner.tasks(buildings_array);

        self.job_runner = Some(job_runner);
    }

    fn insert_building(
        &mut self,
        building: city_data::Building,
        tiles: &BTreeMap<(u32, u32), city_data::Tile>,
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

            self.insert_building(spawn_building, tiles);
            self.spawn_point_encountered.emit((
                Array::from(&[tile_coords.0, tile_coords.1]),
                2,
                altitude,
            ));
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

        let mut location = self.city_coords_feature.get_building_coords(
            tile_coords.0,
            tile_coords.1,
            altitude,
            building_size,
        );

        // fix z fighting of flat buildings
        location.y += 0.1;

        let (_, insert_time) = with_timing(|| {
            self.get_sector(tile_coords)
                .add_child_ex(instance.clone().upcast())
                .force_readable_name(true)
                .done();

            let Some(root) = self
                .base
                .get_tree()
                .and_then(|tree| tree.get_current_scene())
            else {
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
    fn get_sector(&mut self, tile_coords: (u32, u32)) -> Gd<Node3D> {
        const SECTOR_SIZE: u32 = 32;

        let sector_coords = (
            (tile_coords.0 / SECTOR_SIZE) * SECTOR_SIZE,
            (tile_coords.1 / SECTOR_SIZE) * SECTOR_SIZE,
        );

        let sector_name = {
            let (x, y) = sector_coords;

            format!("{}_{}", x, y)
        };

        self.base
            .get_node_or_null(sector_name.to_godot().into())
            .map(Gd::cast)
            .unwrap_or_else(|| {
                let mut sector: Gd<Node3D> = Marker3D::new_alloc().upcast();

                sector.set_name(sector_name.to_godot());

                self.base.add_child(sector.clone().upcast());

                sector.translate(self.city_coords_feature.get_world_coords(
                    sector_coords.0 + (SECTOR_SIZE / 2),
                    sector_coords.1 + (SECTOR_SIZE / 2),
                    0,
                ));

                if let Some(root) = self
                    .base
                    .get_tree()
                    .and_then(|tree| tree.get_current_scene())
                {
                    sector.set_owner(root);
                };

                sector
            })
    }
}

type LocalJob<T, H> = Box<dyn Fn(&mut H, T)>;

#[derive(Dbg)]
struct LocalJobRunner<T, H>
where
    T: Debug,
{
    budget: u64,
    tasks: VecDeque<T>,
    #[dbg(skip)]
    callback: LocalJob<T, H>,
}

impl<T: Debug, H> LocalJobRunner<T, H> {
    fn new<C: Fn(&mut H, T) + 'static>(callback: C, budget: u64) -> Self {
        Self {
            callback: Box::new(callback),
            tasks: VecDeque::new(),
            budget,
        }
    }

    fn poll(&mut self, host: &mut H) -> u32 {
        let start = Time::singleton().get_ticks_msec();
        let mut count = 0;

        while Time::singleton().get_ticks_msec() - start < self.budget {
            let Some(item) = self.tasks.remove(0) else {
                return count;
            };

            (self.callback)(host, item);
            count += 1;
        }

        count
    }

    fn tasks(&mut self, mut tasks: VecDeque<T>) {
        self.tasks.append(&mut tasks);
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
