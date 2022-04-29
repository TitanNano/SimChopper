mod lerp;
mod point;
mod terrain_rotation;
mod tile_surface;
mod ybuffer;

use gdnative::api::{visual_server::ArrayFormat, ArrayMesh, Material, Mesh, SurfaceTool};
use gdnative::prelude::*;

use std::cmp::{max, min};
use std::collections::HashMap;
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

const ERROR_INVALID_VARIANT_TYPE_INT: &str = "Variant is expected to be i64 but is not!";
const ERROR_INVALID_VARIANT_TYPE_ARRAY: &str = "Variant is expected to be VariantArray but is not!";
const ERROR_INVALID_VARIANT_TYPE_OBJECT: &str = "Variant is expected to be an Object but is not!";

struct TileData(Dictionary<Shared>);

impl TileData {
    pub fn new(object: Dictionary) -> Self {
        Self(object)
    }

    fn property(&self, name: &str) -> Variant {
        self.0.get(name).unwrap_or_else(Variant::nil)
    }

    pub fn terrain(&self) -> i64 {
        self.property("terrain")
            .to()
            .expect(ERROR_INVALID_VARIANT_TYPE_INT)
    }

    pub fn altitude(&self) -> i64 {
        self.property("altitude")
            .to()
            .expect(ERROR_INVALID_VARIANT_TYPE_INT)
    }

    pub fn coordinates(&self) -> Vec<i64> {
        let array: VariantArray = self
            .property("coordinates")
            .to()
            .expect(ERROR_INVALID_VARIANT_TYPE_ARRAY);

        array
            .iter()
            .map(|value: Variant| value.to().expect(ERROR_INVALID_VARIANT_TYPE_INT))
            .collect()
    }

    pub fn has_building(&self) -> bool {
        let variant = self.property("building");

        if variant.is_nil() {
            return false;
        }

        let building: Dictionary = variant.to().expect(ERROR_INVALID_VARIANT_TYPE_OBJECT);

        let building_id: i64 = building
            .get("building_id")
            .unwrap_or_else(Variant::nil)
            .to()
            .expect(ERROR_INVALID_VARIANT_TYPE_INT);

        building_id > 0
    }
}

type SurfaceMap = HashMap<TileSurfaceType, Vec<VertexRef>>;

fn stitch_chunk_seams(vertices: Vec<Vec<VertexRef>>) {
    let mut normal_map: HashMap<String, Vector3> = HashMap::new();
    let vertices: Vec<_> = vertices.into_iter().flatten().collect();

    vertices.iter().for_each(|vertex| {
        let vertex = vertex.lock().unwrap();
        let vertex_key = vertex.to_string();

        let normal = {
            let normal = normal_map.get(&vertex_key).unwrap_or(&Vector3::ZERO);

            *normal + *vertex.normal()
        };

        normal_map.insert(vertex_key, normal);
    });

    vertices.into_iter().for_each(|vertex| {
        let mut vertex = vertex.lock().unwrap();
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

            let v0 = v0.lock().unwrap();
            let v1 = v1.lock().unwrap();
            let v2 = v2.lock().unwrap();

            let plane = Plane::from_points(v0.as_vector(), v1.as_vector(), v2.as_vector())
                .expect("points a not all members of the same plane");

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
    let key = VariantArray::new();

    key.push(x as i32);
    key.push(y as i32);

    key.into_shared().to_variant()
}

#[derive(NativeClass)]
#[no_constructor] // disallow default constructor
#[inherit(Reference)]
pub struct TerrainBuilder {
    tile_size: u8,
    city_size: u16,
    tile_height: u8,
    sea_level: u16,
    rotation: Instance<TerrainRotation, Shared>,
    tilelist: Dictionary,
    materials: Dictionary,
}

#[methods]
impl TerrainBuilder {
    fn new(
        rotation: Instance<TerrainRotation, Shared>,
        tilelist: Dictionary,
        materials: Dictionary,
    ) -> Self {
        Self {
            tile_size: 16,
            city_size: 0,
            tile_height: 8,
            sea_level: 0,
            rotation,
            tilelist,
            materials,
        }
    }

    #[export]
    fn set_city_size(&mut self, _base: &Reference, value: u16) {
        self.city_size = value;
    }

    #[export]
    fn set_tile_size(&mut self, _base: &Reference, value: u8) {
        self.tile_size = value;
    }

    #[export]
    fn set_tile_height(&mut self, _base: &Reference, value: u8) {
        self.tile_height = value;
    }

    #[export]
    fn set_sea_level(&mut self, _base: &Reference, value: u16) {
        self.sea_level = value;
    }

    fn add_to_surface(surfaces: &mut SurfaceMap, vertex: Vertex) -> VertexRef {
        let surface = vertex.surface();
        let cell = VertexRef::from(vertex);

        surfaces
            .entry(surface)
            .or_insert_with(Vec::new)
            .push(Arc::clone(&cell));

        cell
    }

    fn tile_vertex_to_city_uv(&self, vertex: &Vertex, tile_size: u8) -> Vector2 {
        let uv_x = vertex.x() / f32::from(self.city_size * tile_size as u16);
        let uv_y = vertex.z() / f32::from(self.city_size * tile_size as u16);

        Vector2::new(uv_x, uv_y)
    }

    fn process_tile(&self, tile_data_dic: Dictionary, edge: TileEdgeType) -> TileFaces {
        let tile_data: TileData = TileData::new(tile_data_dic);
        let tile_type = ((tile_data.terrain() & 0xF0) >> 4) as u8;
        let tile_slope = (tile_data.terrain() & 0x0F) as u8;
        let tile_size = self.tile_size as f32;
        let tile_height = self.tile_height;
        let rotation = &self.rotation;

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
        if tile_type > 0 && (self.sea_level as i64) >= tile_data.altitude() {
            let water_altitude = tile_height as usize * self.sea_level as usize;

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

        tile.apply_slope(tile_slope, rotation, self.tile_height.into());

        let mut tile_faces: Vec<_> = tile.into();
        let mut water_faces = match water {
            Some(water) => water.into(),
            None => vec![],
        };

        tile_faces.append(&mut water_faces);

        tile_faces
    }

    fn build_chunk_vertices(&self, chunk: (u16, u16, u16, u16)) -> (SurfaceMap, Vec<VertexRef>) {
        let mut ybuffer: HashMapYBuffer<VertexRef> = YBuffer::new();
        let mut surfaces = SurfaceMap::new();
        let mut vertices = vec![];
        let mut edge_buffer = vec![];

        let (lower_y, upper_y, lower_x, upper_x) = chunk;

        for y in lower_y..upper_y {
            for x in lower_x..upper_x {
                let is_left_edge = x == lower_x;
                let is_right_edge = x == (upper_x - 1);
                let is_top_edge = y == lower_y;
                let is_bottom_edge = y == (upper_y - 1);
                let edge =
                    TileEdgeType::new(is_top_edge, is_bottom_edge, is_left_edge, is_right_edge);

                let key = create_tilelist_key(x, y);
                let tile = self
                    .tilelist
                    .get(key)
                    .expect("there is a hole in the tilelist!");

                let tile_faces = self.process_tile(tile.to().unwrap(), edge);

                vertices.extend(tile_faces.into_iter().flatten());
            }
        }

        for vertex in vertices {
            let vertex = Self::add_to_surface(&mut surfaces, vertex);

            if vertex.lock().unwrap().is_chunk_edge() {
                edge_buffer.push(vertex.clone());
            }

            if vertex.lock().unwrap().surface() != TileSurfaceType::Water {
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
                let mut vertex = vertex_ref.lock().unwrap();

                let normal = normal_map
                    .get(&vertex.to_string())
                    .unwrap_or_else(|| panic!("no normal for {:?}", vertex))
                    .normalized();

                vertex.set_normal(normal);
            });

        (surfaces, edge_buffer)
    }

    fn build_terain_chunk(&self, surfaces: SurfaceMap) -> Ref<ArrayMesh> {
        let generator = SurfaceTool::new();
        let mesh = ArrayMesh::new();
        let mut vertex_count = 0;

        // all vertices are added to the y buffer and their surfaces
        // we now have to merge the y positions in the y buffer
        // and afterwards we can generate all the surfaces
        for (surface_type, surface) in surfaces {
            generator.clear();
            generator.begin(Mesh::PRIMITIVE_TRIANGLES);
            generator.add_smooth_group(true);

            for vertex_cell in surface {
                let vertex = Arc::try_unwrap(vertex_cell)
                    .expect("too many refs to vertex")
                    .into_inner()
                    .expect("mutex got poisoned");

                generator.add_uv(self.tile_vertex_to_city_uv(&vertex, self.tile_size));
                generator.add_normal(*vertex.normal());

                if vertex.is_chunk_edge() {
                    generator.add_color(Color::from_rgb(1.0, 0.0, 0.0));
                } else {
                    generator.add_color(Color::from_rgb(1.0, 1.0, 1.0));
                }

                generator.add_vertex(vertex.into());

                vertex_count += 1;
            }

            generator.index();
            generator.generate_tangents();

            let surface_arrays = generator.commit_to_arrays();
            let new_index = mesh.get_surface_count();

            mesh.add_surface_from_arrays(
                Mesh::PRIMITIVE_TRIANGLES,
                surface_arrays,
                VariantArray::new_shared(),
                ArrayFormat::COMPRESS_DEFAULT.into(),
            );

            let surface_material_variant = self.materials.get(surface_type.to_string());

            let surface_material: Option<Ref<Material>> = match surface_material_variant {
                Some(material) => material.to_object(),
                None => None,
            };

            match surface_material {
                Some(material) => mesh.surface_set_material(new_index, material),
                None => godot_error!("no material for surface type {}", surface_type),
            };
        }

        godot_print!("generated {} vertices for terain", vertex_count);
        godot_print!("done generating surfaces {}ms", 0.0);
        mesh.into_shared()
    }

    #[export]
    pub fn build_terain_async(&self, _base: &Reference) -> Vec<Ref<ArrayMesh>> {
        let chunk_size = 16;

        // we need to be certain that we have a compatible city size
        assert!((self.city_size % chunk_size) == 0);

        let chunk_count = self.city_size / (chunk_size as u16);
        let timer = Instant::now();
        let mut chunks: Vec<(u16, u16, u16, u16)> = Vec::with_capacity(chunk_count as usize);

        for chunk_y in 0..chunk_count {
            for chunk_x in 0..chunk_count {
                let chunk_lower_y = max(chunk_y * chunk_size, 0);
                let chunk_upper_y = min((chunk_y + 1) * chunk_size, self.city_size);
                let chunk_lower_x = max(chunk_x * chunk_size, 0);
                let chunk_upper_x = min((chunk_x + 1) * chunk_size, self.city_size);

                chunks.push((chunk_lower_y, chunk_upper_y, chunk_lower_x, chunk_upper_x));
            }
        }

        let (surface_chunks, chunk_edge_vertices): (Vec<_>, Vec<_>) = chunks
            .into_par_iter()
            .map(|chunk| self.build_chunk_vertices(chunk))
            .unzip();

        stitch_chunk_seams(chunk_edge_vertices);

        let result = surface_chunks
            .into_par_iter()
            .map(|chunk| self.build_terain_chunk(chunk))
            .collect();

        godot_print!("terrain build time: {}ms", timer.elapsed().as_millis());

        result
    }
}

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct TerrainBuilderFactory;

#[methods]
impl TerrainBuilderFactory {
    fn new(_base: &Reference) -> Self {
        Self
    }

    #[export]
    pub fn create(
        &self,
        _base: &Reference,
        tilelist: Dictionary,
        rotation: Instance<TerrainRotation, Shared>,
        materials: Dictionary,
    ) -> Instance<TerrainBuilder, Unique> {
        let builder = TerrainBuilder::new(rotation, tilelist, materials);

        builder.emplace()
    }
}
