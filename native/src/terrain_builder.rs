use gdnative::api::{visual_server::ArrayFormat, ArrayMesh, Material, Mesh, SurfaceTool};
use gdnative::prelude::*;
use lerp::Lerp;

use core::hash::Hash;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

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

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
enum TileSurfaceType {
    Ground,
    Water,
}

impl fmt::Display for TileSurfaceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Self::Ground => "Ground",
            Self::Water => "Water",
        };

        write!(f, "{}", value)
    }
}

#[derive(Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    surface: TileSurfaceType,
    fixed: bool,
}

impl Vertex {
    fn new(surface: TileSurfaceType, x: f32, y: f32, z: f32, fixed: bool) -> Self {
        Self {
            surface,
            x,
            y,
            z,
            fixed,
        }
    }

    fn from_vector(surface: TileSurfaceType, vector: Vector3, fixed: bool) -> Self {
        Self::new(surface, vector.x, vector.y, vector.z, fixed)
    }
}

impl Into<Vector3> for Vertex {
    fn into(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

trait DimensionX {
    fn x(&self) -> f32;
}

trait DimensionZ {
    fn z(&self) -> f32;
}

trait DimensionY {
    fn y(&self) -> f32;
}

trait SetDimensionY {
    fn set_y(self, value: f32);
}

trait FixedPoint {
    fn is_fixed(&self) -> bool;
}

trait GetMut<T> {
    fn get_mut(&self) -> &mut T;
}

impl DimensionX for Vertex {
    fn x(&self) -> f32 {
        self.x
    }
}

impl<V: DimensionX> DimensionX for Rc<V> {
    fn x(&self) -> f32 {
        (**self).x()
    }
}

impl<V: DimensionX> DimensionX for RefCell<V> {
    fn x(&self) -> f32 {
        self.borrow().x()
    }
}

impl DimensionZ for Vertex {
    fn z(&self) -> f32 {
        self.z
    }
}

impl<V: DimensionZ> DimensionZ for Rc<V> {
    fn z(&self) -> f32 {
        (**self).z()
    }
}

impl<V: DimensionZ> DimensionZ for RefCell<V> {
    fn z(&self) -> f32 {
        self.borrow().z()
    }
}

impl DimensionY for Vertex {
    fn y(&self) -> f32 {
        self.y
    }
}

impl<V: DimensionY> DimensionY for Rc<V> {
    fn y(&self) -> f32 {
        (**self).y()
    }
}

impl<V: DimensionY> DimensionY for RefCell<V> {
    fn y(&self) -> f32 {
        self.borrow().y()
    }
}

impl SetDimensionY for &mut Vertex {
    fn set_y(self, value: f32) {
        self.y = value;
    }
}

impl SetDimensionY for Vertex {
    fn set_y(mut self, value: f32) {
        self.y = value;
    }
}

impl<V> SetDimensionY for Rc<V>
where
    for<'a> &'a V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        (*self).set_y(value)
    }
}

impl<V> SetDimensionY for &RefCell<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.borrow_mut().set_y(value);
    }
}

impl<V> SetDimensionY for RefCell<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.borrow_mut().set_y(value);
    }
}

impl FixedPoint for Vertex {
    fn is_fixed(&self) -> bool {
        self.fixed
    }
}

impl<V: FixedPoint> FixedPoint for Rc<V> {
    fn is_fixed(&self) -> bool {
        (**self).is_fixed()
    }
}

impl<V: FixedPoint> FixedPoint for RefCell<V> {
    fn is_fixed(&self) -> bool {
        self.borrow().is_fixed()
    }
}

type TileCorners = [Vector3; 4];
type Face = [Vertex; 3];
type SurfaceMap = HashMap<TileSurfaceType, Vec<Rc<RefCell<Vertex>>>>;
type TileFaces = (Vec<Face>, Vec<Face>);
type HashMapYBuffer<Value> = HashMap<(usize, usize), Vec<Value>>;

trait YBuffer<Value: DimensionX + DimensionZ + DimensionY + FixedPoint + SetDimensionY>: Sized {
    fn add(&mut self, value: Value);
    fn new() -> Self;
    fn into_iter_groups(self) -> Box<dyn Iterator<Item = Vec<Value>>>;

    fn reduce(self) {
        for vertex_group in self.into_iter_groups() {
            let count: usize = vertex_group.len();
            let peak_y = vertex_group
                .iter()
                .filter(|v| v.is_fixed())
                .map(|v| v.y())
                .reduce(f32::max)
                .unwrap_or(0.0);

            let average_y = if peak_y > 0.0 {
                peak_y
            } else {
                let total_y: f32 = vertex_group.iter().map(|v| v.y()).sum();

                total_y / (count as f32)
            };

            for vertex in vertex_group {
                vertex.set_y(average_y);
            }
        }
    }
}

impl<'v, V: 'static + DimensionX + DimensionZ + DimensionY + FixedPoint + SetDimensionY> YBuffer<V>
    for HashMapYBuffer<V>
{
    fn add(&mut self, value: V) {
        let xz = (value.x().round() as usize, value.z().round() as usize);

        if !self.contains_key(&xz) {
            self.insert(xz, vec![]);
        }

        self.get_mut(&xz)
            .expect("we just made sure that the key is set!")
            .push(value)
    }

    fn new() -> Self {
        Self::new()
    }

    fn into_iter_groups(self) -> Box<dyn Iterator<Item = Vec<V>>> {
        Box::new(self.into_iter().map(|(_, value)| value))
    }
}

fn bilerp<T: Lerp<F> + Copy, F: Copy>(points: [T; 4], weight_x: F, weight_y: F) -> T {
    let x = points[0].lerp(points[1], weight_x);
    let y = points[2].lerp(points[3], weight_x);

    x.lerp(y, weight_y)
}

fn bilerp_xyz(points: &[Vector3; 4], x: f32, y: f32) -> Vector3 {
    let target_x = bilerp(points.map(|v| v.x), x, y);
    let target_y = bilerp(points.map(|v| v.y), x, y);
    let target_z = bilerp(points.map(|v| v.z), x, y);

    Vector3::new(target_x, target_y, target_z)
}

#[derive(Clone)]
struct TileSurface {
    kind: TileSurfaceType,
    corners: TileCorners,
    resolution: u8,
    fixed: bool,
}

impl TileSurface {
    fn new(kind: TileSurfaceType) -> Self {
        Self {
            kind,
            corners: [Vector3::ZERO, Vector3::ZERO, Vector3::ZERO, Vector3::ZERO],
            resolution: 2,
            fixed: false,
        }
    }

    fn resolution(&self) -> u8 {
        self.resolution
    }

    fn set_resolution(&mut self, value: u8) {
        self.resolution = value
    }

    fn corners(&self) -> &TileCorners {
        &self.corners
    }

    fn set_corners(&mut self, value: TileCorners) {
        self.corners = value
    }

    fn kind(&self) -> TileSurfaceType {
        self.kind
    }

    fn set_fixed(&mut self, fixed: bool) {
        self.fixed = fixed;
    }

    fn generate_faces(&self) -> Vec<Face> {
        let mut faces: Vec<Face> = Vec::new();
        let kind = self.kind();
        let resolution = self.resolution();

        for ix in 0..resolution {
            let weight_x_start = 1.0 / (resolution as f32) * (ix as f32);
            let weight_x_end = 1.0 / (resolution as f32) * ((ix as f32) + 1.0);

            for iy in 0..resolution {
                let corners = self.corners();
                let weight_y_start = 1.0 / (resolution as f32) * (iy as f32);
                let weight_y_end = 1.0 / (resolution as f32) * ((iy as f32) + 1.0);

                let x0 = bilerp_xyz(corners, weight_x_start, weight_y_start);
                let x1 = bilerp_xyz(corners, weight_x_end, weight_y_start);
                let y0 = bilerp_xyz(corners, weight_x_start, weight_y_end);
                let y1 = bilerp_xyz(corners, weight_x_end, weight_y_end);

                faces.append(&mut vec![
                    [
                        Vertex::from_vector(kind, x0, self.fixed),
                        Vertex::from_vector(kind, x1, self.fixed),
                        Vertex::from_vector(kind, y1, self.fixed),
                    ],
                    [
                        Vertex::from_vector(kind, x0, self.fixed),
                        Vertex::from_vector(kind, y1, self.fixed),
                        Vertex::from_vector(kind, y0, self.fixed),
                    ],
                ])
            }
        }

        return faces;
    }
}

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
        let surface = vertex.surface;
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
        let uv_x = vertex.x / f32::from(self.city_size * tile_size as u16);
        let uv_y = vertex.z / f32::from(self.city_size * tile_size as u16);

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
            water_tile.kind = TileSurfaceType::Water;
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
            water_tile.kind = TileSurfaceType::Water;
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

            if vertex.borrow().surface != TileSurfaceType::Water {
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
