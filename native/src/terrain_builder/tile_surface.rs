use std::fmt::{self, Display};

use godot::prelude::*;

use crate::util::logger;
use crate::world::city_data::TerrainSlope;

use super::lerp::bilerp_xyz;
use super::point::{DimensionX, DimensionY, DimensionZ, FixedPoint, SetDimensionY};
use super::terrain_rotation::TerrainRotation;

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

        write!(f, "{value}")
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
pub(crate) struct TileSurface {
    kind: TileSurfaceType,
    pub corners: TileCorners,
    resolution: u8,
    fixed: bool,
    invalid: bool,
}

impl TileSurface {
    pub fn new(kind: TileSurfaceType) -> Self {
        Self {
            kind,
            corners: [Vector3::ZERO, Vector3::ZERO, Vector3::ZERO, Vector3::ZERO],
            resolution: 2,
            fixed: false,
            invalid: false,
        }
    }

    fn resolution(&self) -> u8 {
        self.resolution
    }

    pub fn set_resolution(&mut self, value: u8) {
        self.resolution = value;
    }

    fn corners(&self) -> &TileCorners {
        &self.corners
    }

    pub fn set_corners(&mut self, value: TileCorners) {
        self.corners = value;
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

    pub fn set_invalid(&mut self, is_invalid: bool) {
        self.invalid = is_invalid;
    }

    #[expect(clippy::too_many_lines)]
    pub fn apply_slope(&mut self, slope: TerrainSlope, rotation: &TerrainRotation, height: f32) {
        match slope {
            TerrainSlope::None => (),
            TerrainSlope::North => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::East => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::South => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::West => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::NorthSouthEast => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::SouthNorthEast => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::SouthNorthWest => {
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::NorthSouthWest => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::NorthEast => {
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::SouthEast => {
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::SouthWest => {
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::NorthWest => {
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::All | TerrainSlope::VertialCliff => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::Undetermined => {
                logger::warn!("The Undetermined terrain slope should not have any corners!");
            }
            TerrainSlope::NorthWestEast2SouthEast => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::NorthEastWest2SouthWest => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::SouthWestEast2NorthEast => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::SouthEastWest2NorthWest => {
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::South2NorthEast => {
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::East2NorthWest => {
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::North2SouthWest => {
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::West2SouthEast => {
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::South2NorthWest => {
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::East2SouthWest => {
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::North2SouthEast => {
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::West2NorthEast => {
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::WestSouthEast2 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height * 2.0;
            }
            TerrainSlope::SouthNorthEast2 => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.ne()].y += height * 2.0;
            }
            TerrainSlope::EastNorthWest2 => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.nw()].y += height * 2.0;
            }
            TerrainSlope::NorthSouthWest2 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.sw()].y += height * 2.0;
            }
            TerrainSlope::NorthWest2East => {
                self.corners[rotation.nw()].y += height * 2.0;
                self.corners[rotation.ne()].y += height;
            }
            TerrainSlope::SouthWest2NorthWest => {
                self.corners[rotation.sw()].y += height * 2.0;
                self.corners[rotation.nw()].y += height;
            }
            TerrainSlope::SouthEast2West => {
                self.corners[rotation.se()].y += height * 2.0;
                self.corners[rotation.sw()].y += height;
            }
            TerrainSlope::NorthEast2SouthEast => {
                self.corners[rotation.ne()].y += height * 2.0;
                self.corners[rotation.se()].y += height;
            }
            TerrainSlope::WestNorthEast2 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.ne()].y += height * 2.0;
            }
            TerrainSlope::SouthNorthWest2 => {
                self.corners[rotation.sw()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.nw()].y += height * 2.0;
            }
            TerrainSlope::EastSouthWest2 => {
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height;
                self.corners[rotation.sw()].y += height * 2.0;
            }
            TerrainSlope::NorthSouthEast2 => {
                self.corners[rotation.nw()].y += height;
                self.corners[rotation.ne()].y += height;
                self.corners[rotation.se()].y += height * 2.0;
            }
        }
    }
}

impl From<TileSurface> for Vec<Face> {
    #[expect(clippy::similar_names)]
    fn from(tile: TileSurface) -> Vec<Face> {
        let mut faces: Vec<Face> = Vec::new();
        let kind = tile.kind();
        let resolution = tile.resolution();

        for ix in 0..resolution {
            let weight_x_start = 1.0 / f32::from(resolution) * f32::from(ix);
            let weight_x_end = 1.0 / f32::from(resolution) * (f32::from(ix) + 1.0);

            for iy in 0..resolution {
                let corners = tile.corners();

                let weight_y_start = 1.0 / f32::from(resolution) * f32::from(iy);
                let weight_y_end = 1.0 / f32::from(resolution) * (f32::from(iy) + 1.0);

                let x0 = bilerp_xyz(corners, weight_x_start, weight_y_start);
                let x1 = bilerp_xyz(corners, weight_x_end, weight_y_start);
                let y0 = bilerp_xyz(corners, weight_x_start, weight_y_end);
                let y1 = bilerp_xyz(corners, weight_x_end, weight_y_end);

                faces.append(&mut vec![
                    [
                        Vertex::from_vector(kind, x0, tile.fixed, tile.invalid),
                        Vertex::from_vector(kind, x1, tile.fixed, tile.invalid),
                        Vertex::from_vector(kind, y1, tile.fixed, tile.invalid),
                    ],
                    [
                        Vertex::from_vector(kind, x0, tile.fixed, tile.invalid),
                        Vertex::from_vector(kind, y1, tile.fixed, tile.invalid),
                        Vertex::from_vector(kind, y0, tile.fixed, tile.invalid),
                    ],
                ]);
            }
        }

        faces
    }
}

/// Surface Vertex that is used as an intermediate representation of a mesh vertex
/// before it is passed to the [`SurfaceTool`].
#[derive(Debug)]
pub(crate) struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    surface: TileSurfaceType,
    fixed: bool,
    is_invalid_tile: bool,
}

impl Vertex {
    fn new(
        surface: TileSurfaceType,
        x: f32,
        y: f32,
        z: f32,
        fixed: bool,
        is_invalid_tile: bool,
    ) -> Self {
        Self {
            x,
            y,
            z,
            surface,
            fixed,
            is_invalid_tile,
        }
    }

    fn from_vector(
        surface: TileSurfaceType,
        vector: Vector3,
        fixed: bool,
        is_invalid_tile: bool,
    ) -> Self {
        Self::new(
            surface,
            vector.x,
            vector.y,
            vector.z,
            fixed,
            is_invalid_tile,
        )
    }

    pub fn is_invalid_tile(&self) -> bool {
        self.is_invalid_tile
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

impl Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}y{}z", self.x, self.y, self.z)
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
