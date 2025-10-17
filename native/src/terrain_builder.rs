mod lerp;
mod point;
mod terrain_rotation;
mod tile_surface;

use std::collections::HashMap;
use std::ops::{Deref, Not};
use std::time::Instant;

use godot::classes::mesh::PrimitiveType;
use godot::classes::{ArrayMesh, Material, SurfaceTool};
use godot::meta::GodotType;
use godot::{prelude::*, task};
use itertools::Itertools;
use kanal::{ReceiveError, Receiver};
use num_enum::TryFromPrimitive;
use rayon::prelude::*;

use point::{DimensionX, DimensionZ};
use tile_surface::{Face, SurfaceAssociated, TileFaces, TileSurface, TileSurfaceType, Vertex};

pub(crate) use terrain_rotation::TerrainRotation;

use crate::objects::scene_object_registry::{Bridge, Powerlines, Road};
use crate::util::async_support::{godot_future, GodotFuture};
use crate::util::logger;
use crate::world::city_data::{
    TerrainSlope, TerrainType, Tile, TileCoords, TileList, TileListExt, TileValidationResult,
    TryFromDictionary,
};

struct Shared<T: GodotType>(T);

unsafe impl<T: GodotType> Send for Shared<T> {}
unsafe impl<T: GodotType> Sync for Shared<T> {}

impl<T: GodotType> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: GodotType> Shared<T> {
    fn inner(&self) -> &T {
        &self.0
    }
}

type SurfaceMap = HashMap<TileSurfaceType, Vec<Vertex>>;

struct ChunkConfig {
    tile_coords: (u32, u32),
    size: u32,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct TerrainChunk {
    config: ChunkConfig,
    mesh: Shared<Gd<ArrayMesh>>,
}

#[godot_api]
impl TerrainChunk {
    #[func]
    pub fn mesh(&self) -> Gd<ArrayMesh> {
        self.mesh.inner().to_owned()
    }

    #[func]
    pub fn tile_coords(&self) -> Array<u32> {
        let (x, y) = self.config.tile_coords;

        Array::from(&[x, y])
    }
}

struct ChunkSurfaces {
    config: ChunkConfig,
    surfaces: SurfaceMap,
}

struct CoordinatorThreadContext {
    tile_size: u8,
    city_size: u32,
    tile_height: u8,
    sea_level: u16,
    chunk_size: u32,
    rotation: TerrainRotation,
    materials: Shared<Dictionary>,
    debug_render_invalid: bool,
    render_water: bool,
}

impl<'a> CoordinatorThreadContext {
    fn to_worker(&'a self, tilelist: &'a TileList) -> WorkerThreadContext<'a> {
        WorkerThreadContext {
            tile_size: self.tile_size,
            city_size: self.city_size,
            tile_height: self.tile_height,
            sea_level: self.sea_level,
            rotation: &self.rotation,
            tilelist,
            materials: &self.materials,
            debug_render_invalid: self.debug_render_invalid,
            render_water: self.render_water,
        }
    }
}

struct WorkerThreadContext<'a> {
    tile_size: u8,
    city_size: u32,
    tile_height: u8,
    sea_level: u16,
    rotation: &'a TerrainRotation,
    tilelist: &'a TileList,
    materials: &'a Shared<Dictionary>,
    debug_render_invalid: bool,
    render_water: bool,
}

impl WorkerThreadContext<'_> {
    fn tile_vertex_to_city_uv(&self, vertex: &Vertex) -> Vector2 {
        let uv_x = vertex.x() / (self.city_size * self.tile_size as u32) as f32;
        let uv_y = vertex.z() / (self.city_size * self.tile_size as u32) as f32;

        Vector2::new(uv_x, uv_y)
    }
}

enum TerrainBuilderProgress {
    Progress,
    Complete(Vec<TerrainChunk>),
}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct TerrainBuilder {
    tile_size: u8,
    city_size: u32,
    tile_height: u8,
    sea_level: u16,
    chunk_size: u32,
    rotation: Gd<TerrainRotation>,
    tilelist: Dictionary,
    materials: Dictionary,
    debug_render_invalid: bool,
    render_water: bool,
    base: Base<RefCounted>,
}

#[godot_api]
impl TerrainBuilder {
    const GROUND_SURFACE: &str = "ground";

    const WATER_SURFACE: &str = "water";

    #[func]
    fn ground_surface() -> StringName {
        Self::GROUND_SURFACE.into()
    }

    #[func]
    fn new(
        tilelist: Dictionary,
        rotation: Gd<TerrainRotation>,
        materials: Dictionary,
    ) -> Gd<TerrainBuilder> {
        Gd::from_init_fn(|base| TerrainBuilder {
            tile_size: 16,
            city_size: 0,
            tile_height: 8,
            sea_level: 0,
            chunk_size: 8,
            rotation,
            tilelist,
            materials,
            debug_render_invalid: false,
            render_water: true,
            base,
        })
    }

    #[func]
    fn set_city_size(&mut self, value: u32) {
        self.city_size = value;
    }

    #[func]
    fn set_tile_size(&mut self, value: u8) {
        self.tile_size = value;
    }

    #[func]
    fn set_tile_height(&mut self, value: u8) {
        self.tile_height = value;
    }

    #[func]
    fn set_sea_level(&mut self, value: u16) {
        self.sea_level = value;
    }

    #[func]
    fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[func]
    fn load_steps(&self) -> u32 {
        (self.city_size / self.chunk_size).pow(2) * self.chunk_size * 2
    }

    #[signal]
    fn progress(count: u32);

    fn materials(&self) -> Shared<Dictionary> {
        Shared(self.materials.clone())
    }

    fn rotation(&self) -> &Gd<TerrainRotation> {
        &self.rotation
    }

    fn add_to_surface(surfaces: &mut SurfaceMap, vertex: Vertex) {
        let surface = vertex.surface();

        surfaces.entry(surface).or_default().push(vertex);
    }

    fn spawn_build_thread(
        context: CoordinatorThreadContext,
        tilelist: TileList,
        chunk_size: u32,
        chunk_count: u32,
    ) -> Receiver<TerrainBuilderProgress> {
        let (tx, rx) = kanal::unbounded::<TerrainBuilderProgress>();

        std::thread::spawn(move || {
            let timer = Instant::now();
            let tilelist = Self::pre_process_tilelist(&context, tilelist);

            let chunks = (0..chunk_count)
                .flat_map(|y| (0..chunk_count).map(move |x| (x, y)))
                .map(|(x, y)| ChunkConfig {
                    tile_coords: (x * chunk_size, y * chunk_size),
                    size: chunk_size,
                });

            let worker_context = context.to_worker(&tilelist);

            let result: Vec<_> = chunks
                .par_bridge()
                .map(|chunk| generate_chunk_vertices(&worker_context, chunk))
                // Send progress updates for each finished chunk
                .inspect(|_| {
                    if let Err(err) = tx.send(TerrainBuilderProgress::Progress) {
                        logger::error!("Failed to send TerrainBuilder progress: {}", err);
                    }
                })
                .map(|chunk| generate_chunk_mesh(&worker_context, chunk))
                // send progress updates for each finished mesh
                .inspect(|_| {
                    if let Err(err) = tx.send(TerrainBuilderProgress::Progress) {
                        logger::error!("Failed to send TerrainBuilder progress: {}", err);
                    }
                })
                .collect();

            godot_print!("terrain build time: {}ms", timer.elapsed().as_millis());

            if let Err(err) = tx.send(TerrainBuilderProgress::Complete(result)) {
                logger::error!("Failed to Send TerrainBuilder final message: {}", err);
            }
        });

        rx
    }

    #[func]
    pub fn build_terain_async(&self) -> Gd<GodotFuture> {
        let chunk_size = self.chunk_size;

        // we need to be certain that we have a compatible city size
        debug_assert!((self.city_size % chunk_size) == 0);

        let chunk_count = self.city_size / chunk_size;
        let rotation = self.rotation().bind().deref().to_owned();
        let tilelist = TileList::try_from_dict(&self.tilelist)
            .expect("TileList passed from GDScript must be valid");
        let context = self.thread_context(rotation);
        let builder: Gd<Self> = self.base().clone().cast();

        let (resolve, future) = godot_future::<Vec<Gd<TerrainChunk>>>();

        task::spawn(async move {
            let receiver = Self::spawn_build_thread(context, tilelist, chunk_size, chunk_count);

            // Async loop that receives progress updates and sends them through godot signals
            loop {
                match receiver.as_async().recv().await {
                    Ok(TerrainBuilderProgress::Progress) => {
                        builder.signals().progress().emit(chunk_size);
                    }
                    Ok(TerrainBuilderProgress::Complete(result)) => {
                        let chunks = result.into_iter().map(Gd::from_object).collect();

                        resolve(chunks);
                        break;
                    }

                    Err(ReceiveError::SendClosed | ReceiveError::Closed) => {
                        logger::error!(
                            "orchestration thread has disconnected before completing terrain!"
                        );
                        break;
                    }
                }
            }
        });

        future
    }

    fn pre_process_tilelist(
        context: &CoordinatorThreadContext,
        mut tilelist: TileList,
    ) -> TileList {
        let rotation = &context.rotation;

        let chunks: Vec<_> = tilelist
            .values()
            .chunks(context.chunk_size.pow(2) as usize)
            .into_iter()
            .map(|chunk| chunk.collect_vec())
            .collect();

        let mut pending_tiles: Vec<_> = chunks
            .par_iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|tile| (*tile, tilelist.validate_tile_slope(tile, rotation)))
                    .filter_map(|(tile, validation_result)| {
                        if !validation_result.is_invalid() {
                            return None;
                        }

                        if let Some(special_case) =
                            is_special_case(tile, &tilelist, &validation_result)
                        {
                            return Some((tile, validation_result, Some(special_case)));
                        }

                        (tile.has_building() || tile.has_surface_water())
                            .not()
                            .then_some((tile, validation_result, None))
                    })
                    .par_bridge()
            })
            .flatten()
            .map(|(tile, invalid, special_case)| (tile.to_owned(), invalid, special_case))
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by(|(_, a, _), (_, b, _)| a.empty_invalid_tiles.cmp(&b.empty_invalid_tiles))
            .collect();

        drop(chunks);

        // remove all the invalid tiles first
        pending_tiles
            .iter_mut()
            .for_each(|(original_tile, _, special_case)| {
                let tile = tilelist.get_mut(&original_tile.coordinates).unwrap();

                tile.terrain.slope = TerrainSlope::Undetermined;

                match special_case {
                    // bridge transition pieces have to be lowered by one level of altitude.
                    // This also applies to power line cliffs
                    Some(TileSpecialCase::BridgeTransition | TileSpecialCase::PowerlineCliff) => {
                        original_tile.altitude -= 1;
                        tile.altitude -= 1;
                    }

                    Some(TileSpecialCase::PintchedAllSlope) => {
                        original_tile.altitude += 1;
                        original_tile.terrain.slope = TerrainSlope::None;
                        tile.altitude += 1;
                    }

                    Some(TileSpecialCase::Powerline) | Some(TileSpecialCase::Bridge) | None => (),
                }
            });

        let invalid_empty_tiles = pending_tiles.len();
        let mut fixed_tiles = 0;

        pending_tiles.into_iter().for_each(|(mut tile, _, _)| {
            let options = tilelist.valid_slopes(&tile, rotation);

            let Some(slope_type) = options.into_iter().next() else {
                tilelist.insert(tile.coordinates, tile);
                return;
            };

            tile.terrain.slope = rotation.to_reverted().normalize_slope(*slope_type);
            tilelist.insert(tile.coordinates, tile);
            fixed_tiles += 1;
        });

        logger::debug!("Fixed {fixed_tiles} of {invalid_empty_tiles} empty tiles");

        tilelist
    }
}

impl TerrainBuilder {
    fn thread_context(&self, rotation: TerrainRotation) -> CoordinatorThreadContext {
        CoordinatorThreadContext {
            tile_size: self.tile_size,
            city_size: self.city_size,
            tile_height: self.tile_height,
            sea_level: self.sea_level,
            chunk_size: self.chunk_size,
            rotation,
            materials: self.materials(),
            debug_render_invalid: self.debug_render_invalid,
            render_water: self.render_water,
        }
    }
}

#[derive(GodotClass)]
#[class(base = RefCounted, init)]
struct TerrainBuilderFactory;

#[godot_api]
impl TerrainBuilderFactory {
    #[func]
    fn create(
        &self,
        tilelist: Dictionary,
        rotation: Gd<TerrainRotation>,
        materials: Dictionary,
    ) -> Gd<TerrainBuilder> {
        TerrainBuilder::new(tilelist, rotation, materials)
    }
}

enum TileSpecialCase {
    BridgeTransition,
    Powerline,
    PowerlineCliff,
    Bridge,
    PintchedAllSlope,
}

fn generate_tile_surfaces(
    context: &WorkerThreadContext,
    tile_data: &Tile,
    tilelist: &TileList,
) -> TileFaces {
    let tile_size = context.tile_size as f32;
    let tile_height = context.tile_height;
    let rotation = context.rotation;
    let coords = tile_data.coordinates();

    // validate tile slope
    let is_invalid_type = context
        .debug_render_invalid
        .then(|| tilelist.validate_tile_slope(tile_data, rotation))
        .is_some_and(|invalid_result| invalid_result.is_invalid());

    let tile_x = (coords.0 * tile_size as u32) as f32;
    let tile_y = (coords.1 * tile_size as u32) as f32;
    let tile_z = (tile_data.altitude() * tile_height as u32) as f32;

    let mut tile_surface = TileSurface::new(TileSurfaceType::Ground);

    tile_surface.set_corners([
        //			0											1
        Vector3::new(tile_x, tile_z, tile_y),
        Vector3::new(tile_x + tile_size, tile_z, tile_y),
        //			2											3
        Vector3::new(tile_x, tile_z, tile_y + tile_size),
        Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
    ]);

    tile_surface.set_fixed(tile_data.has_building());
    tile_surface.set_invalid(is_invalid_type);

    let mut extra_surfaces: Vec<TileSurface> = Vec::new();

    tile_surface.apply_slope(
        tile_data.terrain.slope,
        rotation,
        context.tile_height.into(),
    );

    match tile_data.terrain.ty {
        TerrainType::DryLand => {
            // (-1, 0)
            handle_tile_cliff(
                (tile_data.coordinates.0 - 1, tile_data.coordinates.1),
                [
                    Vector3::new(tile_x, tile_z, tile_y),
                    tile_surface.corners[0],
                    Vector3::new(tile_x, tile_z, tile_y + tile_size),
                    tile_surface.corners[2],
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (+1, 0)
            handle_tile_cliff(
                (tile_data.coordinates.0 + 1, tile_data.coordinates.1),
                [
                    Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
                    tile_surface.corners[3],
                    Vector3::new(tile_x + tile_size, tile_z, tile_y),
                    tile_surface.corners[1],
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (0, -1)
            handle_tile_cliff(
                (tile_data.coordinates.0, tile_data.coordinates.1 - 1),
                [
                    Vector3::new(tile_x + tile_size, tile_z, tile_y),
                    tile_surface.corners[1],
                    Vector3::new(tile_x, tile_z, tile_y),
                    tile_surface.corners[0],
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (0, +1)
            handle_tile_cliff(
                (tile_data.coordinates.0, tile_data.coordinates.1 + 1),
                [
                    Vector3::new(tile_x, tile_z, tile_y + tile_size),
                    tile_surface.corners[2],
                    Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
                    tile_surface.corners[3],
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );
        }

        // tile is covered by water
        TerrainType::Underwater | TerrainType::Shoreline
            if (context.sea_level as u32) >= tile_data.altitude() =>
        {
            let water_altitude = tile_height as usize * context.sea_level as usize;

            let mut water_tile = tile_surface.clone();
            water_tile.set_kind(TileSurfaceType::Water);
            water_tile.set_resolution(3);

            water_tile.corners[0].y = water_altitude as f32;
            water_tile.corners[1].y = water_altitude as f32;
            water_tile.corners[2].y = water_altitude as f32;
            water_tile.corners[3].y = water_altitude as f32;

            if context.render_water {
                extra_surfaces.push(water_tile);
            }
        }

        TerrainType::Underwater | TerrainType::Shoreline => {
            logger::warn!("Tile is Underwater or Shoreline but above sea level!");
        }

        TerrainType::SurfaceWater if tile_data.terrain.slope == TerrainSlope::VertialCliff => {
            let mut water_tile = tile_surface.clone();

            water_tile.set_kind(TileSurfaceType::Water);
            water_tile.set_resolution(3);

            extra_surfaces.push(water_tile);

            tile_surface.corners[rotation.nw()].y -= tile_height as f32;
            tile_surface.corners[rotation.ne()].y -= tile_height as f32;
            tile_surface.corners[rotation.sw()].y -= tile_height as f32;
            tile_surface.corners[rotation.se()].y -= tile_height as f32;

            // (-1, 0)
            handle_water_tile_cliff(
                (tile_data.coordinates.0 - 1, tile_data.coordinates.1),
                [
                    Vector3::new(tile_x, tile_z, tile_y),
                    Vector3::new(tile_x, tile_z + tile_height as f32, tile_y),
                    Vector3::new(tile_x, tile_z, tile_y + tile_size),
                    Vector3::new(tile_x, tile_z + tile_height as f32, tile_y + tile_size),
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (+1, 0)
            handle_water_tile_cliff(
                (tile_data.coordinates.0 + 1, tile_data.coordinates.1),
                [
                    Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
                    Vector3::new(
                        tile_x + tile_size,
                        tile_z + tile_height as f32,
                        tile_y + tile_size,
                    ),
                    Vector3::new(tile_x + tile_size, tile_z, tile_y),
                    Vector3::new(tile_x + tile_size, tile_z + tile_height as f32, tile_y),
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (0, -1)
            handle_water_tile_cliff(
                (tile_data.coordinates.0, tile_data.coordinates.1 - 1),
                [
                    Vector3::new(tile_x + tile_size, tile_z, tile_y),
                    Vector3::new(tile_x + tile_size, tile_z + tile_height as f32, tile_y),
                    Vector3::new(tile_x, tile_z, tile_y),
                    Vector3::new(tile_x, tile_z + tile_height as f32, tile_y),
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );

            // (ß, +1)
            handle_water_tile_cliff(
                (tile_data.coordinates.0, tile_data.coordinates.1 + 1),
                [
                    Vector3::new(tile_x, tile_z, tile_y + tile_size),
                    Vector3::new(tile_x, tile_z + tile_height as f32, tile_y + tile_size),
                    Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
                    Vector3::new(
                        tile_x + tile_size,
                        tile_z + tile_height as f32,
                        tile_y + tile_size,
                    ),
                ],
                tile_data,
                tilelist,
                &mut extra_surfaces,
            );
        }

        // tile is surface water
        TerrainType::SurfaceWater | TerrainType::MoreSurfaceWater => {
            let mut water_tile = tile_surface.clone();
            water_tile.set_kind(TileSurfaceType::Water);
            water_tile.set_resolution(3);

            water_tile.corners[rotation.nw()].y = tile_z;
            water_tile.corners[rotation.ne()].y = tile_z;
            water_tile.corners[rotation.sw()].y = tile_z;
            water_tile.corners[rotation.se()].y = tile_z;

            extra_surfaces.push(water_tile);

            tile_surface.corners[rotation.nw()].y -= tile_height as f32;
            tile_surface.corners[rotation.ne()].y -= tile_height as f32;
            tile_surface.corners[rotation.sw()].y -= tile_height as f32;
            tile_surface.corners[rotation.se()].y -= tile_height as f32
        }
    }

    if matches!(tile_data.terrain.slope, TerrainSlope::Undetermined) {
        return vec![];
    }

    let mut tile_faces: Vec<_> = tile_surface.into();
    let water_faces = extra_surfaces.into_iter().flat_map(Vec::<Face>::from);

    tile_faces.extend(water_faces);

    tile_faces
}

fn generate_chunk_vertices(context: &WorkerThreadContext, chunk: ChunkConfig) -> ChunkSurfaces {
    let mut surfaces = SurfaceMap::new();

    let lower_y = chunk.tile_coords.1;
    let lower_x = chunk.tile_coords.0;
    let upper_y = lower_y + chunk.size;
    let upper_x = lower_x + chunk.size;

    for y in lower_y..upper_y {
        for x in lower_x..upper_x {
            let key = (x, y);
            let tile = context
                .tilelist
                .get(&key)
                .expect("there is a hole in the tilelist!");

            generate_tile_surfaces(context, tile, context.tilelist)
                .into_iter()
                .flatten()
                .for_each(|vertex| {
                    TerrainBuilder::add_to_surface(&mut surfaces, vertex);
                });
        }
    }

    ChunkSurfaces {
        surfaces,
        config: chunk,
    }
}

/// Generate an ArrayMesh from a list of surface vertecies.
fn generate_chunk_mesh(context: &WorkerThreadContext, chunk: ChunkSurfaces) -> TerrainChunk {
    let mut generator = SurfaceTool::new_gd();
    let mut mesh = ArrayMesh::new_gd();
    let mut vertex_count = 0;

    for (surface_type, surface) in chunk.surfaces {
        generator.clear();
        generator.begin(PrimitiveType::TRIANGLES);

        // calculate global offset. Vertex contains the wold coordinates and we have to subtract the
        // offset to get the model coordinates.
        let world_offset = Vector3 {
            x: (chunk.config.tile_coords.0 * u32::from(context.tile_size)) as f32,
            y: 0.0,
            z: (chunk.config.tile_coords.1 * u32::from(context.tile_size)) as f32,
        };

        for vertex in surface {
            generator.set_uv(context.tile_vertex_to_city_uv(&vertex));

            let smooth_group = match surface_type {
                TileSurfaceType::Ground => u32::MAX,
                TileSurfaceType::Water => 0,
            };

            generator.set_smooth_group(smooth_group);
            generator.set_color(
                if context.debug_render_invalid && vertex.is_invalid_tile() {
                    Color::RED
                } else {
                    Color::BLACK
                },
            );
            generator.add_vertex(Vector3::from(vertex) - world_offset);

            vertex_count += 1;
        }

        generator.index();
        generator.generate_normals();
        generator.generate_tangents();

        let surface_arrays = generator.commit_to_arrays();
        let new_index = mesh.get_surface_count();

        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &surface_arrays);

        let surface_name = match surface_type {
            TileSurfaceType::Ground => TerrainBuilder::GROUND_SURFACE,
            TileSurfaceType::Water => TerrainBuilder::WATER_SURFACE,
        };

        mesh.surface_set_name(new_index, surface_name);

        let surface_material_variant = context.materials.get(surface_type.to_string());

        let surface_material: Option<Gd<Material>> =
            surface_material_variant.map(|material| material.to());

        match surface_material {
            Some(material) => mesh.surface_set_material(new_index, &material),
            None => logger::error!("no material for surface type {}", surface_type),
        };
    }

    logger::info!("generated {} vertices for terain", vertex_count);

    TerrainChunk {
        mesh: Shared(mesh),
        config: chunk.config,
    }
}

/// Tile is a special case and requires some additional transformation.
fn is_special_case(
    tile: &Tile,
    tilelist: &TileList,
    validation_result: &TileValidationResult,
) -> Option<TileSpecialCase> {
    let terrain = tile.terrain.slope;
    let building_id = tile.building.as_ref().map(|building| building.building_id);

    match (terrain, building_id, validation_result.invalid_tiles) {
        // Terrain slope is the start of a bridge.
        (
            TerrainSlope::South | TerrainSlope::North | TerrainSlope::West | TerrainSlope::East,
            Some(building_id),
            _,
        ) if Road::try_from_primitive(building_id).is_ok() => tilelist
            .get_tile_neighbors(tile)
            .filter_map(|(_, neighbor)| neighbor.building.as_ref())
            .any(|neighbor| Bridge::try_from_primitive(neighbor.building_id).is_ok())
            .then_some(TileSpecialCase::BridgeTransition),

        // Terrain slope is raised in all corners and stuck between exactly two neighbors with two corners.
        (TerrainSlope::All, None, 6) => Some(TileSpecialCase::PintchedAllSlope),

        (
            TerrainSlope::South | TerrainSlope::North | TerrainSlope::West | TerrainSlope::East,
            Some(building_id),
            _,
        ) if Powerlines::try_from_primitive(building_id).is_ok() => tilelist
            .get_tile_neighbors(tile)
            .all(|(_, neighbor)| neighbor.altitude <= tile.altitude)
            .then_some(TileSpecialCase::PowerlineCliff),

        (_, Some(building_id), _) if Powerlines::try_from_primitive(building_id).is_ok() => {
            Some(TileSpecialCase::Powerline)
        }

        (_, Some(building_id), _) if Bridge::try_from_primitive(building_id).is_ok() => {
            Some(TileSpecialCase::Bridge)
        }

        _ => None,
    }
}

/// Detects two neighbors that are invalid and create a 90 deg cliff.
fn is_invalid_cliff(tile: &Tile, neighbor: &Tile) -> bool {
    let invalid_slope = matches!(
        tile.terrain.slope,
        TerrainSlope::West | TerrainSlope::South | TerrainSlope::East | TerrainSlope::North
    ) && (tile.altitude == neighbor.altitude
        && neighbor.terrain.slope == TerrainSlope::None
        || tile.altitude > neighbor.altitude && neighbor.terrain.slope == TerrainSlope::All);

    let invalid_plateau =
        matches!(tile.terrain.slope, TerrainSlope::All) && tile.altitude == neighbor.altitude;

    invalid_slope || invalid_plateau
}

#[inline]
fn handle_tile_cliff(
    neighbor_coords: TileCoords,
    cliff_corners: [Vector3; 4],
    tile: &Tile,
    tilelist: &TileList,
    surfaces: &mut Vec<TileSurface>,
) {
    let neighbor = tilelist.get(&neighbor_coords);

    if !neighbor.is_some_and(|neighbor| is_invalid_cliff(tile, neighbor)) {
        return;
    }

    let mut cliff_side = TileSurface::new(TileSurfaceType::Ground);
    cliff_side.set_resolution(1);
    cliff_side.set_corners(cliff_corners);

    surfaces.push(cliff_side);
}

#[inline]
fn handle_water_tile_cliff(
    neighbor_coords: TileCoords,
    cliff_corners: [Vector3; 4],
    tile_data: &Tile,
    tilelist: &TileList,
    surfaces: &mut Vec<TileSurface>,
) {
    let neighbor = tilelist.get(&neighbor_coords);

    if !neighbor.is_some_and(|neighbor| is_water_cliff(tile_data, neighbor)) {
        return;
    }

    let mut water_side = TileSurface::new(TileSurfaceType::Water);

    water_side.set_resolution(3);
    water_side.set_corners(cliff_corners);

    surfaces.push(water_side);
}

/// Detects a waterfall cliff.
#[inline]
fn is_water_cliff(tile: &Tile, neighbor: &Tile) -> bool {
    neighbor.altitude < tile.altitude
        || neighbor.altitude == tile.altitude
            && (!matches!(
                neighbor.terrain.ty,
                TerrainType::SurfaceWater | TerrainType::MoreSurfaceWater
            ) || !matches!(neighbor.terrain.slope, TerrainSlope::VertialCliff))
}
