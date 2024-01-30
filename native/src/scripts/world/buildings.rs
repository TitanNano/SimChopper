use std::{collections::VecDeque, fmt::Debug, ops::Not};

use derive_debug::Dbg;
use godot::{
    builtin::{meta::ToGodot, Array, Dictionary, VariantArray},
    engine::{utilities::snappedi, Node, Node3D, NodeExt, Resource, Time},
    log::{godot_error, godot_print},
    obj::{Gd, NewAlloc},
};
use godot_rust_script::{godot_script_impl, GodotScript, ScriptSignal, Signal};

use crate::{info, objects::scene_object_registry, world::city_coords_feature::CityCoordsFeature};

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct Buildings {
    #[export]
    pub world_constants: Option<Gd<Resource>>,

    city_coords_feature: CityCoordsFeature,
    job_runner: Option<LocalJobRunner<Dictionary, Self>>,

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
    pub fn _ready(&mut self) {}

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

    pub fn build_async(&mut self, city: Dictionary) {
        let simulator_settings: Dictionary = city.get_or_nil("simulator_settings").to();
        let sea_level: u32 = simulator_settings.get_or_nil("GlobalSeaLevel").to();
        let buildings: Dictionary = city.get_or_nil("buildings").to();
        let tiles: Dictionary = city.get_or_nil("tilelist").to();

        self.city_coords_feature = CityCoordsFeature::new(
            self.world_constants
                .as_ref()
                .expect("world_constants should be set!")
                .to_owned(),
            sea_level,
        );

        info!("starting to load buildings...");

        let mut job_runner = LocalJobRunner::new(
            move |host: &mut Self, building: Dictionary| {
                if building.get_or_nil("building_id").to::<u8>() == 0x00 {
                    info!("skipping empty building");
                    return;
                }

                host.insert_building(building, &tiles);
            },
            50,
        );

        let buildings_array = buildings
            .values_array()
            .iter_shared()
            .map(|variant| variant.to())
            .collect();

        job_runner.tasks(buildings_array);

        self.job_runner = Some(job_runner);
    }

    fn insert_building(&mut self, building: Dictionary, tiles: &Dictionary) {
        let building_size: u8 = building
            .get("size")
            .expect("insert_building: missing size")
            .to();
        let name: String = building
            .get("name")
            .expect("insert_building: missing name")
            .to();
        let building_id: u8 = building
            .get("building_id")
            .expect("insert_building: missing building_id")
            .to();
        let object = scene_object_registry::load_building(building_id);
        let tile_coords: Array<u32> = building
            .get("tile_coords")
            .expect("insert_building: missing tile_coords")
            .to::<VariantArray>()
            .iter_shared()
            .map(|value| value.to())
            .collect();
        let tile: Dictionary = tiles
            .get(tile_coords.clone())
            .expect("insert_building: missing tile")
            .to();
        let altitude: u32 = tile
            .get("altitude")
            .expect("insert_building: missing altitude")
            .to();

        let Some(object) = object else {
            godot_error!("unknown building \"{}\"", name);
            return;
        };

        if building_id == scene_object_registry::Buildings::Tarmac
            && is_spawn_point(&building, tiles)
        {
            info!("encountered a spawn point: {:?}", building);
            let mut spawn_building = Dictionary::new();

            spawn_building.set(
                "building_id",
                scene_object_registry::Buildings::Hangar2 as u8,
            );

            spawn_building.set(
                "tile_coords",
                tile_coords
                    .iter_shared()
                    .map(|v| v.to_variant())
                    .collect::<VariantArray>(),
            );
            spawn_building.set("name", "Hangar");
            spawn_building.set("size", 2);
            spawn_building.set("altitude", altitude);

            self.insert_building(spawn_building, tiles);
            self.spawn_point_encountered
                .emit((tile_coords.clone(), 2, altitude));
        }

        let (Some(instance), instance_time) = with_timing(|| object.instantiate()) else {
            godot_error!("failed to instantiate building {}", name);
            return;
        };

        let mut location = self.city_coords_feature.get_building_coords(
            tile_coords.get(0),
            tile_coords.get(1),
            altitude,
            building_size,
        );

        // fix z fighting of flat buildings
        location.y += 0.1;

        let sector_name = {
            let x = snappedi(tile_coords.get(0) as f64, 10);
            let y = snappedi(tile_coords.get(1) as f64, 10);

            format!("{}_{}", x, y)
        };

        let (_, insert_time) = with_timing(|| {
            self.get_sector(sector_name)
                .add_child_ex(instance.clone())
                .force_readable_name(true)
                .done()
        });

        let (_, translate_time) = with_timing(|| instance.cast::<Node3D>().translate(location));

        if instance_time > 100 {
            godot_error!("\"{}\" is very slow to instantiate!", name);
        }

        if insert_time > 100 {
            godot_error!("\"{}\" is very slow to insert!", name);
        }

        if translate_time > 100 {
            godot_error!("\"{}\" is very slow to translate!", name);
        }
    }

    fn get_sector(&mut self, name: String) -> Gd<Node> {
        self.base
            .get_node_or_null(name.to_godot().into())
            .unwrap_or_else(|| {
                let mut sector = Node::new_alloc();

                sector.set_name(name.to_godot());

                self.base.add_child(sector);
                self.base.get_node_as(name)
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

fn is_spawn_point(building: &Dictionary, tiles: &Dictionary) -> bool {
    let tile_coords: Array<u8> = building
        .get_or_nil("tile_coords")
        .to::<VariantArray>()
        .iter_shared()
        .map(|item| item.to())
        .collect();
    let x = tile_coords.get(0);
    let y = tile_coords.get(1);

    let x_miss = (x - 1..x + 3)
        .all(|index| {
            let Some(tile) = tiles.get(VariantArray::from(&[index.to_variant(), y.to_variant()]))
            else {
                godot_error!("unable to get tile at: x = {}, y = {}", index, y);
                return false;
            };

            let tile = tile.to::<Dictionary>();

            tile.get("building")
                .and_then(|building| building.try_to::<Dictionary>().ok())
                .and_then(|building| building.get("building_id"))
                .map(|id| id.to::<u8>() == scene_object_registry::Buildings::Tarmac)
                .unwrap_or(false)
        })
        .not();

    if x_miss {
        return false;
    }

    (y - 1..y + 3).all(|index| {
        tiles
            .get(VariantArray::from(&[x.to_variant(), index.to_variant()]))
            .map(|tile| tile.to::<Dictionary>())
            .and_then(|tile| tile.get("building"))
            .and_then(|building| building.try_to::<Dictionary>().ok())
            .and_then(|building| building.get("building_id"))
            .map(|id| id.to::<u8>() == scene_object_registry::Buildings::Tarmac)
            .unwrap_or(false)
    })
}

fn with_timing<R>(cb: impl FnOnce() -> R) -> (R, u64) {
    let start = Time::singleton().get_ticks_msec();

    let result = cb();

    (result, Time::singleton().get_ticks_msec() - start)
}
