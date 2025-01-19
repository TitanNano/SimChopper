mod lerp;
mod point;
mod terrain_rotation;
mod tile_surface;
mod ybuffer;

use godot::classes::mesh::PrimitiveType;
use godot::classes::{ArrayMesh, Material, SurfaceTool};
use godot::meta::GodotType;
use godot::prelude::*;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use itertools::Itertools;
use rayon::prelude::*;

use point::{DimensionX, DimensionZ};
use terrain_rotation::TerrainRotationBehaviour;
use tile_surface::{
    SurfaceAssociated, TileEdgeType, TileFaces, TileSurface, TileSurfaceType, Vertex, VertexRef,
};
use ybuffer::{HashMapYBuffer, YBuffer};

pub use terrain_rotation::TerrainRotation;

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

struct TileData(Dictionary);

impl TileData {
    pub fn new(object: Dictionary) -> Self {
        Self(object)
    }

    fn property(&self, name: &str) -> Variant {
        self.0.get(name).unwrap_or_else(Variant::nil)
    }

    pub fn terrain(&self) -> i64 {
        self.property("terrain").to()
    }

    pub fn altitude(&self) -> i64 {
        self.property("altitude").to()
    }

    pub fn coordinates(&self) -> Vec<i64> {
        let array: VariantArray = self.property("coordinates").to();

        array
            .iter_shared()
            .map(|value: Variant| value.to())
            .collect()
    }

    pub fn has_building(&self) -> bool {
        let variant = self.property("building");

        if variant.is_nil() {
            return false;
        }

        let building: Dictionary = variant.to();

        let building_id: i64 = building
            .get("building_id")
            .unwrap_or_else(Variant::nil)
            .to();

        building_id > 0
    }
}

type SurfaceMap = HashMap<TileSurfaceType, Vec<VertexRef>>;

fn stitch_chunk_seams(vertices: Vec<Vec<VertexRef>>) {
    let mut normal_map: HashMap<String, Vector3> = HashMap::new();
    let vertices: Vec<_> = vertices.into_iter().flatten().collect();

    vertices.iter().for_each(|vertex| {
        let vertex = vertex.read().unwrap();
        let vertex_key = vertex.to_string();

        let normal = {
            let normal = normal_map.get(&vertex_key).unwrap_or(&Vector3::ZERO);

            *normal + *vertex.normal()
        };

        normal_map.insert(vertex_key, normal);
    });

    vertices.into_iter().for_each(|vertex| {
        let mut vertex = vertex.write().unwrap();
        let normal = normal_map.get(&vertex.to_string()).unwrap();

        vertex.set_normal(normal.normalized());
    });
}

fn calculate_normals<'a, I: Iterator<Item = &'a VertexRef>>(
    vertices: I,
) -> HashMap<String, Vector3> {
    let normal_map: HashMap<String, Vector3> = HashMap::new();

    vertices
        .chunks(3)
        .into_iter()
        .fold(normal_map, |mut normal_map, face| {
            let [v0, v1, v2]: [&VertexRef; 3] =
                face.collect::<Vec<_>>().as_slice().try_into().unwrap();

            let v0 = v0.read().unwrap();
            let v1 = v1.read().unwrap();
            let v2 = v2.read().unwrap();

            let plane = Plane::from_points(v0.as_vector(), v1.as_vector(), v2.as_vector());

            let v0_normal = *normal_map.get(&v0.to_string()).unwrap_or(&Vector3::ZERO);

            normal_map.insert(v0.to_string(), v0_normal + plane.normal);

            let v1_normal = *normal_map.get(&v1.to_string()).unwrap_or(&Vector3::ZERO);

            normal_map.insert(v1.to_string(), v1_normal + plane.normal);

            let v2_normal = *normal_map.get(&v2.to_string()).unwrap_or(&Vector3::ZERO);

            normal_map.insert(v2.to_string(), v2_normal + plane.normal);

            normal_map
        })
}

fn create_tilelist_key(x: u16, y: u16) -> Variant {
    let mut key = Array::new();

    key.push(x as i32);
    key.push(y as i32);

    key.to_variant()
}

struct ChunkConfig {
    tile_coords: (u16, u16),
    size: u16,
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
    pub fn tile_coords(&self) -> Array<u16> {
        let (x, y) = self.config.tile_coords;

        Array::from(&[x, y])
    }
}

struct ChunkSurfaces {
    config: ChunkConfig,
    surfaces: SurfaceMap,
}

struct ThreadContext<'a> {
    tile_size: u8,
    city_size: u16,
    tile_height: u8,
    sea_level: u16,
    rotation: &'a TerrainRotation,
    tilelist: Shared<Dictionary>,
    materials: Shared<Dictionary>,
}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct TerrainBuilder {
    tile_size: u8,
    city_size: u16,
    tile_height: u8,
    sea_level: u16,
    chunk_size: u16,
    rotation: Gd<TerrainRotation>,
    tilelist: Dictionary,
    materials: Dictionary,
}

#[godot_api]
impl TerrainBuilder {
    #[func]
    fn new(
        tilelist: Dictionary,
        rotation: Gd<TerrainRotation>,
        materials: Dictionary,
    ) -> Gd<TerrainBuilder> {
        Gd::from_object(TerrainBuilder {
            tile_size: 16,
            city_size: 0,
            tile_height: 8,
            sea_level: 0,
            chunk_size: 32,
            rotation,
            tilelist,
            materials,
        })
    }

    #[func]
    fn set_city_size(&mut self, value: u16) {
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
    fn chunk_size(&self) -> u16 {
        self.chunk_size
    }

    fn tilelist(&self) -> Shared<Dictionary> {
        Shared(self.tilelist.clone())
    }

    fn materials(&self) -> Shared<Dictionary> {
        Shared(self.materials.clone())
    }

    fn rotation(&self) -> &Gd<TerrainRotation> {
        &self.rotation
    }

    fn add_to_surface(surfaces: &mut SurfaceMap, vertex: Vertex) -> VertexRef {
        let surface = vertex.surface();
        let cell = VertexRef::from(vertex);

        surfaces.entry(surface).or_default().push(Arc::clone(&cell));

        cell
    }

    fn tile_vertex_to_city_uv(context: &ThreadContext, vertex: &Vertex, tile_size: u8) -> Vector2 {
        let uv_x = vertex.x() / f32::from(context.city_size * tile_size as u16);
        let uv_y = vertex.z() / f32::from(context.city_size * tile_size as u16);

        Vector2::new(uv_x, uv_y)
    }

    fn process_tile(
        context: &ThreadContext,
        tile_data_dic: Dictionary,
        edge: TileEdgeType,
    ) -> TileFaces {
        let tile_data: TileData = TileData::new(tile_data_dic);
        let tile_type = ((tile_data.terrain() & 0xF0) >> 4) as u8;
        let tile_slope = (tile_data.terrain() & 0x0F) as u8;
        let tile_size = context.tile_size as f32;
        let tile_height = context.tile_height;
        let rotation = context.rotation;
        //		assert((tile_type > 0 && tileData.is_water) || (tile_type == 0 && not tileData.is_water))

        let tile_x = (tile_data.coordinates()[0] * tile_size as i64) as f32;
        let tile_y = (tile_data.coordinates()[1] * tile_size as i64) as f32;
        let tile_z = (tile_data.altitude() * tile_height as i64) as f32;

        let mut tile = TileSurface::new(TileSurfaceType::Ground, edge);

        tile.set_corners([
            //			0											1
            Vector3::new(tile_x, tile_z, tile_y),
            Vector3::new(tile_x + tile_size, tile_z, tile_y),
            //			2											3
            Vector3::new(tile_x, tile_z, tile_y + tile_size),
            Vector3::new(tile_x + tile_size, tile_z, tile_y + tile_size),
        ]);

        tile.set_fixed(tile_data.has_building());

        let mut water: Option<TileSurface> = None;

        // tile is covered by water
        if tile_type > 0 && (context.sea_level as i64) >= tile_data.altitude() {
            let water_altitude = tile_height as usize * context.sea_level as usize;

            let mut water_tile = tile.clone();
            water_tile.set_kind(TileSurfaceType::Water);
            water_tile.set_resolution(3);

            water_tile.corners[0].y = water_altitude as f32;
            water_tile.corners[1].y = water_altitude as f32;
            water_tile.corners[2].y = water_altitude as f32;
            water_tile.corners[3].y = water_altitude as f32;

            water = Some(water_tile);
        }

        // tile is surface water
        if tile_type == 3 {
            let mut water_tile = tile.clone();
            water_tile.set_kind(TileSurfaceType::Water);
            water_tile.set_resolution(3);

            water = Some(water_tile);

            tile.corners[rotation.nw()].y -= tile_height as f32;
            tile.corners[rotation.ne()].y -= tile_height as f32;
            tile.corners[rotation.sw()].y -= tile_height as f32;
            tile.corners[rotation.se()].y -= tile_height as f32;
        }

        tile.apply_slope(tile_slope, rotation, context.tile_height.into());

        let mut tile_faces: Vec<_> = tile.into();
        let mut water_faces = match water {
            Some(water) => water.into(),
            None => vec![],
        };

        tile_faces.append(&mut water_faces);

        tile_faces
    }

    fn build_chunk_vertices(
        context: &ThreadContext,
        chunk: ChunkConfig,
    ) -> (ChunkSurfaces, Vec<VertexRef>) {
        let mut ybuffer: HashMapYBuffer<VertexRef> = YBuffer::new();
        let mut surfaces = SurfaceMap::new();
        let mut vertices = vec![];
        let mut edge_buffer = vec![];

        let lower_y = chunk.tile_coords.1;
        let lower_x = chunk.tile_coords.0;
        let upper_y = lower_y + chunk.size;
        let upper_x = lower_x + chunk.size;

        for y in lower_y..upper_y {
            for x in lower_x..upper_x {
                let is_left_edge = x == lower_x;
                let is_right_edge = x == (upper_x - 1);
                let is_top_edge = y == lower_y;
                let is_bottom_edge = y == (upper_y - 1);
                let edge =
                    TileEdgeType::new(is_top_edge, is_bottom_edge, is_left_edge, is_right_edge);

                let key = create_tilelist_key(x, y);
                let tile = context
                    .tilelist
                    .get(key)
                    .expect("there is a hole in the tilelist!");

                let tile_faces = Self::process_tile(context, tile.to(), edge);

                vertices.extend(tile_faces.into_iter().flatten());
            }
        }

        for vertex in vertices {
            let vertex = Self::add_to_surface(&mut surfaces, vertex);

            if vertex.read().unwrap().is_chunk_edge() {
                edge_buffer.push(vertex.clone());
            }

            if vertex.read().unwrap().surface() != TileSurfaceType::Water {
                ybuffer.add(vertex);
                continue;
            }
        }

        godot_print!("ybuffer size: {}", ybuffer.len());
        ybuffer.reduce();

        let normal_map = calculate_normals(surfaces.iter().flat_map(|(_, value)| value));

        surfaces
            .iter()
            .flat_map(|(_, value)| value)
            .for_each(|vertex_ref| {
                let mut vertex = vertex_ref.write().unwrap();

                let normal = normal_map
                    .get(&vertex.to_string())
                    .unwrap_or_else(|| panic!("no normal for {:?}", vertex))
                    .to_owned();

                let normal = Vector3::new(normal.x, normal.y, normal.z).normalized();

                vertex.set_normal(normal);
            });

        (
            ChunkSurfaces {
                surfaces,
                config: chunk,
            },
            edge_buffer,
        )
    }

    fn build_terain_chunk(context: &ThreadContext, chunk: ChunkSurfaces) -> TerrainChunk {
        let mut generator = SurfaceTool::new_gd();
        let mut mesh = ArrayMesh::new_gd();
        let mut vertex_count = 0;

        for (surface_type, surface) in chunk.surfaces {
            generator.clear();
            generator.begin(PrimitiveType::TRIANGLES);

            // calculate global offset. Vertex contains the wold coordinates and we have to subtract the
            // offset to get the model coordinates.
            let world_offset = Vector3 {
                x: f32::from(chunk.config.tile_coords.0 * u16::from(context.tile_size)),
                y: 0.0,
                z: f32::from(chunk.config.tile_coords.1 * u16::from(context.tile_size)),
            };

            for vertex_cell in surface {
                let vertex = Arc::try_unwrap(vertex_cell)
                    .expect("too many refs to vertex")
                    .into_inner()
                    .expect("mutex got poisoned");

                generator.set_uv(Self::tile_vertex_to_city_uv(
                    context,
                    &vertex,
                    context.tile_size,
                ));
                generator.set_normal(*vertex.normal());

                if vertex.is_chunk_edge() {
                    generator.set_color(Color::from_rgb(1.0, 0.0, 0.0));
                } else {
                    generator.set_color(Color::from_rgb(1.0, 1.0, 1.0));
                }

                generator.add_vertex(Vector3::from(vertex) - world_offset);

                vertex_count += 1;
            }

            generator.index();
            generator.generate_tangents();

            let surface_arrays = generator.commit_to_arrays();
            let new_index = mesh.get_surface_count();

            mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &surface_arrays);

            let surface_material_variant = context.materials.get(surface_type.to_string());

            let surface_material: Option<Gd<Material>> =
                surface_material_variant.map(|material| material.to());

            match surface_material {
                Some(material) => mesh.surface_set_material(new_index, &material),
                None => godot_error!("no material for surface type {}", surface_type),
            };
        }

        godot_print!("generated {} vertices for terain", vertex_count);
        godot_print!("done generating surfaces {}ms", 0.0);

        TerrainChunk {
            mesh: Shared(mesh),
            config: chunk.config,
        }
    }

    #[func]
    pub fn build_terain_async(&self) -> Array<Gd<TerrainChunk>> {
        let chunk_size = self.chunk_size;

        // we need to be certain that we have a compatible city size
        assert!((self.city_size % chunk_size) == 0);

        let chunk_count = self.city_size / chunk_size;
        let timer = Instant::now();

        let chunks: Vec<_> = (0..chunk_count)
            .flat_map(|y| (0..chunk_count).map(move |x| (x, y)))
            .map(|(x, y)| ChunkConfig {
                tile_coords: (x * chunk_size, y * chunk_size),
                size: chunk_size,
            })
            .collect();

        let rotation = self.rotation().bind();
        let context = self.thread_context(rotation.deref());

        let (surface_chunks, chunk_edge_vertices): (Vec<_>, Vec<_>) = chunks
            .into_par_iter()
            .map(|chunk| Self::build_chunk_vertices(&context, chunk))
            .unzip();

        stitch_chunk_seams(chunk_edge_vertices);

        let result: Vec<_> = surface_chunks
            .into_par_iter()
            .map(|chunk| Self::build_terain_chunk(&context, chunk))
            .collect();

        godot_print!("terrain build time: {}ms", timer.elapsed().as_millis());

        result.into_iter().map(Gd::from_object).collect()
    }
}

impl TerrainBuilder {
    fn thread_context<'a>(&self, rotation: &'a TerrainRotation) -> ThreadContext<'a> {
        ThreadContext {
            tile_size: self.tile_size,
            city_size: self.city_size,
            tile_height: self.tile_height,
            sea_level: self.sea_level,
            rotation,
            tilelist: self.tilelist(),
            materials: self.materials(),
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
