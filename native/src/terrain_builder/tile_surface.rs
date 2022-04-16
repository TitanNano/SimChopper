use std::fmt;

use gdnative::prelude::{Vector3, Instance, Shared};

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

/// The location of all four corners of a tile.
type TileCorners = [Vector3; 4];

/// A face of three vertecies
pub type Face = [Vertex; 3];

/// Group of tile faces for multiple surfaces.
/// A tile can have a ground surface and a water surface.
pub type TileFaces = (Vec<Face>, Vec<Face>);

/// intermediate tile surface meta data that will be used
/// to compute the terrain geometry for a single tile.
#[derive(Clone)]
pub struct TileSurface {
    kind: TileSurfaceType,
    pub corners: TileCorners,
    resolution: u8,
    fixed: bool,
}

impl TileSurface {
    pub fn new(kind: TileSurfaceType) -> Self {
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

    pub fn generate_faces(&self) -> Vec<Face> {
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

    pub fn apply_slope(
        &mut self,
        slope: u8,
        rotation: &Instance<TerrainRotation, Shared>,
        height: f32
    ) {
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

/// Surface Vertex that is used as an intermediate representation of a mesh vertex
/// before it is passed to the SurfaceTool.
#[derive(Debug)]
pub struct Vertex {
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

pub trait SurfaceAssociated {
    fn surface(&self) -> TileSurfaceType;
}

impl SurfaceAssociated for Vertex {
    fn surface(&self) -> TileSurfaceType {
        self.surface
    }
}
