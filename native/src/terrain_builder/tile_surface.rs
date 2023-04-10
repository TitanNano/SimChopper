use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

use godot::prelude::*;

use super::lerp::bilerp_xyz;
use super::point::{DimensionX, DimensionY, DimensionZ, FixedPoint, SetDimensionY};
use super::terrain_rotation::{TerrainRotation, TerrainRotationBehaviour};

/// the type of terrain surface.
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum TileSurfaceType {
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

#[derive(Clone, Debug)]
pub struct TileEdgeType {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl TileEdgeType {
    pub fn new(top: bool, bottom: bool, left: bool, right: bool) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn is_top(&self) -> bool {
        self.top
    }

    pub fn is_bottom(&self) -> bool {
        self.bottom
    }

    pub fn is_left(&self) -> bool {
        self.left
    }

    pub fn is_right(&self) -> bool {
        self.right
    }
}

/// The location of all four corners of a tile.
type TileCorners = [Vector3; 4];

/// A face of three vertecies
pub type Face = [Vertex; 3];

/// Group of tile faces for multiple surfaces.
/// A tile can have a ground surface and a water surface.
pub type TileFaces = Vec<Face>;

/// intermediate tile surface meta data that will be used
/// to compute the terrain geometry for a single tile.
#[derive(Clone)]
pub struct TileSurface {
    kind: TileSurfaceType,
    pub corners: TileCorners,
    resolution: u8,
    edge: TileEdgeType,
    fixed: bool,
}

impl TileSurface {
    pub fn new(kind: TileSurfaceType, edge: TileEdgeType) -> Self {
        Self {
            kind,
            corners: [Vector3::ZERO, Vector3::ZERO, Vector3::ZERO, Vector3::ZERO],
            resolution: 2,
            fixed: false,
            edge,
        }
    }

    fn resolution(&self) -> u8 {
        self.resolution
    }

    pub fn set_resolution(&mut self, value: u8) {
        self.resolution = value
    }

    fn corners(&self) -> &TileCorners {
        &self.corners
    }

    pub fn set_corners(&mut self, value: TileCorners) {
        self.corners = value
    }

    fn kind(&self) -> TileSurfaceType {
        self.kind
    }

    pub fn set_kind(&mut self, value: TileSurfaceType) {
        self.kind = value;
    }

    pub fn set_fixed(&mut self, fixed: bool) {
        self.fixed = fixed;
    }

    pub fn apply_slope(&mut self, slope: u8, rotation: &TerrainRotation, height: f32) {
        match slope {
            0x00 => (),

            0x01 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
            }

            0x02 => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
            }

            0x03 => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
            }

            0x04 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.sw()].y += height;
            }

            0x05 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
            }

            0x06 => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height;
            }

            0x07 => {
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.nw()].y += height;
            }

            0x08 => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
            }

            0x09 => {
                self.corners[rotation.ne()].y += height;
            }

            0x0A => {
                self.corners[rotation.se()].y += height;
            }

            0x0B => {
                self.corners[rotation.sw()].y += height;
            }

            0x0C => {
                self.corners[rotation.nw()].y += height;
            }

            0x0D => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
            }

            _ => {}
        };
    }
}

impl From<TileSurface> for Vec<Face> {
    fn from(tile: TileSurface) -> Vec<Face> {
        let mut faces: Vec<Face> = Vec::new();
        let kind = tile.kind();
        let resolution = tile.resolution();

        for ix in 0..resolution {
            let weight_x_start = 1.0 / (resolution as f32) * (ix as f32);
            let weight_x_end = 1.0 / (resolution as f32) * ((ix as f32) + 1.0);

            for iy in 0..resolution {
                let corners = tile.corners();
                let weight_y_start = 1.0 / (resolution as f32) * (iy as f32);
                let weight_y_end = 1.0 / (resolution as f32) * ((iy as f32) + 1.0);

                let x0 = bilerp_xyz(corners, weight_x_start, weight_y_start);
                let x1 = bilerp_xyz(corners, weight_x_end, weight_y_start);
                let y0 = bilerp_xyz(corners, weight_x_start, weight_y_end);
                let y1 = bilerp_xyz(corners, weight_x_end, weight_y_end);

                let x0_edge = (x0.x == corners[0].x && tile.edge.is_left())
                    || (x0.z == corners[0].z && tile.edge.is_top());

                let x1_edge = (x1.x == corners[1].x && tile.edge.is_right())
                    || (x1.z == corners[1].z && tile.edge.is_top());

                let y0_edge = (y0.x == corners[2].x && tile.edge.is_left())
                    || (y0.z == corners[2].z && tile.edge.is_bottom());

                let y1_edge = (y1.x == corners[3].x && tile.edge.is_right())
                    || (y1.z == corners[3].z && tile.edge.is_bottom());

                faces.append(&mut vec![
                    [
                        Vertex::from_vector(kind, x0, x0_edge, tile.fixed),
                        Vertex::from_vector(kind, x1, x1_edge, tile.fixed),
                        Vertex::from_vector(kind, y1, y1_edge, tile.fixed),
                    ],
                    [
                        Vertex::from_vector(kind, x0, x0_edge, tile.fixed),
                        Vertex::from_vector(kind, y1, y1_edge, tile.fixed),
                        Vertex::from_vector(kind, y0, y0_edge, tile.fixed),
                    ],
                ])
            }
        }

        faces
    }
}

/// Surface Vertex that is used as an intermediate representation of a mesh vertex
/// before it is passed to the SurfaceTool.
#[derive(Debug)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    normal: Vector3,
    surface: TileSurfaceType,
    fixed: bool,
    is_edge: bool,
}

impl Vertex {
    fn new(surface: TileSurfaceType, x: f32, y: f32, z: f32, is_edge: bool, fixed: bool) -> Self {
        Self {
            surface,
            x,
            y,
            z,
            normal: Vector3::ZERO,
            is_edge,
            fixed,
        }
    }

    fn from_vector(surface: TileSurfaceType, vector: Vector3, is_edge: bool, fixed: bool) -> Self {
        Self::new(surface, vector.x, vector.y, vector.z, is_edge, fixed)
    }

    pub fn is_chunk_edge(&self) -> bool {
        self.is_edge
    }

    pub fn as_vector(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    pub fn set_normal(&mut self, value: Vector3) {
        self.normal = value;
    }

    pub fn normal(&self) -> &Vector3 {
        &self.normal
    }
}

impl From<Vertex> for Vector3 {
    fn from(value: Vertex) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl DimensionX for Vertex {
    fn x(&self) -> f32 {
        self.x
    }
}

impl DimensionZ for Vertex {
    fn z(&self) -> f32 {
        self.z
    }
}

impl DimensionY for Vertex {
    fn y(&self) -> f32 {
        self.y
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

impl FixedPoint for Vertex {
    fn is_fixed(&self) -> bool {
        self.fixed
    }
}

impl ToString for Vertex {
    fn to_string(&self) -> String {
        format!("{}x{}y{}z", self.x, self.y, self.z)
    }
}

pub trait SurfaceAssociated {
    fn surface(&self) -> TileSurfaceType;
}

impl SurfaceAssociated for Vertex {
    fn surface(&self) -> TileSurfaceType {
        self.surface
    }
}

pub type VertexRef = Arc<Mutex<Vertex>>;

impl From<Vertex> for VertexRef {
    fn from(value: Vertex) -> Self {
        Arc::new(Mutex::new(value))
    }
}
