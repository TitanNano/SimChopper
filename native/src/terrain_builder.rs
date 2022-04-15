mod lerp;
mod point;
mod tile_surface;
mod ybuffer;

use gdnative::api::{visual_server::ArrayFormat, ArrayMesh, Material, Mesh, SurfaceTool};
use gdnative::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use point::{DimensionX, DimensionZ};
use tile_surface::{SurfaceAssociated, TileFaces, TileSurface, TileSurfaceType, Vertex};
use ybuffer::{YBuffer, HashMapYBuffer};

const ERROR_CLASS_INSTANCE_ACCESS: &str = "unable to access NativeClass instance!";
const ERROR_INVALID_VARIANT_TYPE_INT: &str = "Variant is expected to be i64 but is not!";
const ERROR_INVALID_VARIANT_TYPE_ARRAY: &str = "Variant is expected to be VariantArray but is not!";
const ERROR_INVALID_VARIANT_TYPE_OBJECT: &str = "Variant is expected to be an Object but is not!";

struct TileData(Dictionary<Shared>);

impl TileData {
    pub fn new(object: Dictionary) -> Self {
        Self(object)
    }

    fn property(&self, name: &str) -> Variant {
        self.0.get(name).unwrap_or(Variant::nil())
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
            .unwrap_or(Variant::nil())
            .to()
            .expect(ERROR_INVALID_VARIANT_TYPE_INT);

        building_id > 0
    }
}

type SurfaceMap = HashMap<TileSurfaceType, Vec<Rc<RefCell<Vertex>>>>;

const TERAIN_ROTATION_CORNERS: [u8; 4] = [0, 1, 3, 2];

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct TerrainRotation {
    offset: u8,
}

#[methods]
impl TerrainRotation {
    fn new(_base: &Reference) -> Self {
        Self { offset: 0 }
    }

    #[export]
    fn set_rotation(&mut self, _base: &Reference, rotation: i64) {
        self.offset = u8::try_from(rotation).unwrap_or(u8::MAX);
    }
}

trait TerrainRotationBehaviour {
    fn get_corner(&self, index: u8) -> u8;

    fn nw(&self) -> usize;
    fn ne(&self) -> usize;
    fn se(&self) -> usize;
    fn sw(&self) -> usize;
}

impl TerrainRotationBehaviour for TerrainRotation {
    fn get_corner(&self, index: u8) -> u8 {
        let shifted_index = ((index + self.offset) % 4) as usize;
        let target_value = TERAIN_ROTATION_CORNERS.get(shifted_index).unwrap_or(&0);

        return target_value.to_owned();
    }

    fn nw(&self) -> usize {
        self.get_corner(0).into()
    }

    fn ne(&self) -> usize {
        return self.get_corner(1).into();
    }

    fn se(&self) -> usize {
        return self.get_corner(2).into();
    }

    fn sw(&self) -> usize {
        return self.get_corner(3).into();
    }
}

impl TerrainRotationBehaviour for Instance<TerrainRotation, Shared> {
    fn get_corner(&self, index: u8) -> u8 {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.get_corner(index))
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn nw(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.nw())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn ne(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.ne())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn se(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.se())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn sw(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.sw())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }
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

    fn apply_tile_slope(
        &self,
        tile: &mut TileSurface,
        slope: u8,
        rotation: &Instance<TerrainRotation, Shared>,
    ) {
        let height: f32 = self.tile_height.into();

        match slope {
            0x00 => (),

            0x01 => {
                tile.corners[rotation.nw()].y += height;
                tile.corners[rotation.ne()].y += height;
            }

            0x02 => {
                tile.corners[rotation.ne()].y += height;
                tile.corners[rotation.se()].y += height;
            }

            0x03 => {
                tile.corners[rotation.sw()].y += height;
                tile.corners[rotation.se()].y += height;
            }

            0x04 => {
                tile.corners[rotation.nw()].y += height;
                tile.corners[rotation.sw()].y += height;
            }

            0x05 => {
                tile.corners[rotation.nw()].y += height;
                tile.corners[rotation.ne()].y += height;
                tile.corners[rotation.se()].y += height;
            }

            0x06 => {
                tile.corners[rotation.ne()].y += height;
                tile.corners[rotation.se()].y += height;
                tile.corners[rotation.sw()].y += height;
            }

            0x07 => {
                tile.corners[rotation.se()].y += height;
                tile.corners[rotation.sw()].y += height;
                tile.corners[rotation.nw()].y += height;
            }

            0x08 => {
                tile.corners[rotation.sw()].y += height;
                tile.corners[rotation.nw()].y += height;
                tile.corners[rotation.ne()].y += height;
            }

            0x09 => {
                tile.corners[rotation.ne()].y += height;
            }

            0x0A => {
                tile.corners[rotation.se()].y += height;
            }

            0x0B => {
                tile.corners[rotation.sw()].y += height;
            }

            0x0C => {
                tile.corners[rotation.nw()].y += height;
            }

            0x0D => {
                tile.corners[rotation.nw()].y += height;
                tile.corners[rotation.ne()].y += height;
                tile.corners[rotation.sw()].y += height;
                tile.corners[rotation.se()].y += height;
            }

            _ => {}
        };
    }

    fn add_to_surface<'m, 'v>(surfaces: &'m mut SurfaceMap, vertex: Vertex) -> Rc<RefCell<Vertex>> {
        let surface = vertex.surface();
        let cell = Rc::new(RefCell::new(vertex));

        if !surfaces.contains_key(&surface) {
            surfaces.insert(surface, vec![]);
        }

        surfaces
            .get_mut(&surface)
            .expect("we just made sure that the key is set!")
            .push(Rc::clone(&cell));

        cell
    }

    fn tile_vertex_to_city_uv(&self, vertex: &Vertex, tile_size: u8) -> Vector2 {
        let uv_x = vertex.x() / f32::from(self.city_size * tile_size as u16);
        let uv_y = vertex.z() / f32::from(self.city_size * tile_size as u16);

        return Vector2::new(uv_x, uv_y);
    }

    fn process_tile(&self, tile_data_dic: Dictionary) -> TileFaces {
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

        let mut tile = TileSurface::new(TileSurfaceType::Ground);

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

        self.apply_tile_slope(&mut tile, tile_slope, &rotation);

        let tile_faces = tile.generate_faces();
        let water_faces = match water {
            Some(water) => water.generate_faces(),
            None => vec![],
        };

        return (tile_faces, water_faces);
    }

    #[export]
    pub fn build_terain_async(&self, _base: &Reference) -> Ref<ArrayMesh> {
        let mut ybuffer: HashMapYBuffer<Rc<RefCell<Vertex>>> = YBuffer::new();
        let mut surfaces = SurfaceMap::new();

        let terrain_faces: Vec<TileFaces> = self
            .tilelist
            .values()
            .iter()
            .map(|tile: Variant| self.process_tile(tile.to().unwrap()))
            .collect();

        let vertecies = terrain_faces
            .into_iter()
            .flat_map(|faces: TileFaces| {
                let (tile_faces, water_faces) = faces;

                tile_faces.into_iter().chain(water_faces.into_iter())
            })
            .flatten();

        for vertex in vertecies {
            let vertex = Self::add_to_surface(&mut surfaces, vertex);

            if vertex.borrow().surface() != TileSurfaceType::Water {
                ybuffer.add(vertex);
                continue;
            }
        }

        godot_print!("ybuffer size: {}", ybuffer.len());
        ybuffer.reduce();

        let generator = SurfaceTool::new();
        let mesh = ArrayMesh::new();
        let mut vertex_count = 0;

        // all vertecies are added to the y buffer and their surfaces
        // we now have to merge the y positions in the y buffer
        // and afterwards we can generate all the surfaces
        for (surface_type, surface) in surfaces {
            generator.clear();
            generator.begin(Mesh::PRIMITIVE_TRIANGLES);
            generator.add_smooth_group(true);

            for vertex_cell in surface {
                let vertex = Rc::try_unwrap(vertex_cell)
                    .expect("too many refs to vertex")
                    .into_inner();

                generator.add_uv(self.tile_vertex_to_city_uv(&vertex, self.tile_size));
                generator.add_vertex(vertex.into());

                vertex_count += 1;
            }

            generator.index();
            generator.generate_normals(false);
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

        godot_print!("generated {} vertecies for terain", vertex_count);
        godot_print!("done generating surfaces {}ms", 0.0);
        return mesh.into_shared();
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
