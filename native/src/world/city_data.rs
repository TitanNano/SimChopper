use std::collections::{BTreeMap, HashSet};

use godot::builtin::{Dictionary, VariantArray};
use godot::global::godot_warn;
use godot::meta::error::ConvertError;
use godot::meta::FromGodot;

use crate::objects::scene_object_registry::Buildings;
use crate::terrain_builder::TerrainRotation;

mod terrain_slope;

pub(crate) use terrain_slope::*;

pub(crate) trait TryFromDictionary: Sized {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TryFromDictError {
    #[error("dictionary key \"{0}\" is missing")]
    MissingKey(&'static str),
    #[error("dictionary key \"{0}\" has an unexpected type")]
    InvalidType(Box<str>, #[source] ErasedConvertError),
    #[error(transparent)]
    InvalidKey(ErasedConvertError),
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub(crate) struct ErasedConvertError {
    message: String,
}

impl From<ConvertError> for ErasedConvertError {
    fn from(value: ConvertError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Compass {
    North,
    East,
    South,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

fn get_valid_tile_slopes_from_neighbors<'a>(
    tile: &'a Tile,
    rotation: TerrainRotation,
    tilelist: &'a TileList,
) -> impl Iterator<Item = (Compass, &'a Tile, HashSet<&'static TerrainSlope>)> {
    tilelist
        .get_tile_neighbors(tile)
        .map(move |(dir, neighbor_tile)| {
            let offset = (i64::from(tile.altitude) - i64::from(neighbor_tile.altitude))
                .try_into()
                .unwrap_or(i8::MIN);

            let options = match dir {
                Compass::North => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_south_neighbors(),
                Compass::East => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_west_neighbors(),
                Compass::South => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_north_neighbors(),
                Compass::West => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_east_neighbor(),

                Compass::NorthWest => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_south_east_neighbors(),
                Compass::NorthEast => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_south_west_neighbors(),
                Compass::SouthWest => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_north_east_neighbors(),
                Compass::SouthEast => rotation
                    .normalize_slope(neighbor_tile.terrain.slope)
                    .valid_north_west_neighbors(),
            };

            let available_options = options
                .iter()
                .filter_map(|(option, alt)| (*alt == offset).then_some(option))
                .collect::<HashSet<_>>();

            (dir, neighbor_tile, available_options)
        })
}

pub type TileList = BTreeMap<TileCoords, Tile>;

pub(crate) trait TileListExt {
    fn get_tile_neighbors<'a>(&'a self, tile: &Tile) -> impl Iterator<Item = (Compass, &'a Tile)>;

    /// Validate the slope of a tile. Returns the number of neighbors that do not fit with this tile.
    fn validate_tile_slope(&self, tile: &Tile, rotation: TerrainRotation) -> TileValidationResult;

    fn valid_slopes(
        &self,
        tile: &Tile,
        rotation: TerrainRotation,
    ) -> HashSet<&'static TerrainSlope>;
}

#[derive(Default, Debug)]
pub(crate) struct TileValidationResult {
    pub invalid_tiles: u8,
    pub empty_invalid_tiles: u8,
}

impl TileValidationResult {
    pub fn is_invalid(&self) -> bool {
        self.invalid_tiles > 0
    }
}

impl TileListExt for TileList {
    fn get_tile_neighbors<'a>(&'a self, tile: &Tile) -> impl Iterator<Item = (Compass, &'a Tile)> {
        let coords = tile.coordinates;
        [
            (Compass::South, (coords.0, coords.1.wrapping_sub(1))),
            (Compass::East, (coords.0.wrapping_sub(1), coords.1)),
            (Compass::North, (coords.0, coords.1 + 1)),
            (Compass::West, (coords.0 + 1, coords.1)),
            (Compass::NorthWest, (coords.0 + 1, coords.1 + 1)),
            (Compass::NorthEast, (coords.0.wrapping_sub(1), coords.1 + 1)),
            (Compass::SouthWest, (coords.0 + 1, coords.1.wrapping_sub(1))),
            (
                Compass::SouthEast,
                (coords.0.wrapping_sub(1), coords.1.wrapping_sub(1)),
            ),
        ]
        .into_iter()
        .filter_map(|(dir, coords)| self.get(&coords).map(|coords| (dir, coords)))
    }
    fn validate_tile_slope(&self, tile: &Tile, rotation: TerrainRotation) -> TileValidationResult {
        let neighbors = get_valid_tile_slopes_from_neighbors(tile, rotation, self);

        neighbors
            .filter(|(_, _, valid_options)| {
                !valid_options.contains(&rotation.normalize_slope(tile.terrain.slope))
            })
            .fold(
                TileValidationResult::default(),
                |mut result, (_, neighbor_tile, _)| {
                    result.invalid_tiles += 1;
                    result.empty_invalid_tiles += u8::from(!neighbor_tile.has_building());
                    result
                },
            )
    }

    fn valid_slopes(
        &self,
        tile: &Tile,
        rotation: TerrainRotation,
    ) -> HashSet<&'static TerrainSlope> {
        let neighbors = get_valid_tile_slopes_from_neighbors(tile, rotation, self);

        neighbors
            .map(|(_, _, options)| options)
            .reduce(|a, b| a.intersection(&b).copied().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub(crate) struct City {
    pub simulator_settings: SimulatorSettings,
    pub buildings: BTreeMap<TileCoords, Building>,
    pub tilelist: TileList,
}

impl TryFromDictionary for City {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            simulator_settings: get_dict_key(value, "simulator_settings")
                .and_then(|value| SimulatorSettings::try_from_dict(&value))?,

            buildings: get_dict_key(value, "buildings")
                .and_then(|value| BTreeMap::try_from_dict(&value))?,
            tilelist: get_dict_key(value, "tilelist")
                .and_then(|value| BTreeMap::try_from_dict(&value))?,
        })
    }
}

pub type TileCoords = (u32, u32);

#[derive(Debug, Clone)]
pub(crate) struct Building {
    pub size: u8,
    pub name: String,
    pub id: u8,
    pub tile_coords: TileCoords,
}

impl TryFromDictionary for Building {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            size: get_dict_key(value, "size")?,
            name: get_dict_key(value, "name")?,
            id: get_dict_key(value, "building_id")?,
            tile_coords: get_dict_key(value, "tile_coords").and_then(|val| array_to_tuple(&val))?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    DryLand,
    Underwater,
    Shoreline,
    SurfaceWater,
    MoreSurfaceWater,
}

impl From<u8> for TerrainType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::DryLand,
            1 => Self::Underwater,
            2 => Self::Shoreline,
            3 => Self::SurfaceWater,
            4 => Self::MoreSurfaceWater,
            5.. => {
                godot_warn!("TileType is out of range!");

                Self::MoreSurfaceWater
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TileTerrainInfo {
    pub ty: TerrainType,
    pub slope: TerrainSlope,
}

impl From<u32> for TileTerrainInfo {
    fn from(value: u32) -> Self {
        let ty = TerrainType::from(((value & 0xF0) >> 4) as u8);
        let slope =
            TerrainSlope::try_from((value & 0x0F) as u8).expect("TerrainSlope is out of range");

        Self { ty, slope }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Tile {
    pub terrain: TileTerrainInfo,
    pub altitude: u32,
    pub building: Option<Building>,
    pub coordinates: TileCoords,
}

impl Tile {
    pub fn altitude(&self) -> u32 {
        self.altitude
    }

    pub fn coordinates(&self) -> TileCoords {
        self.coordinates
    }

    pub fn has_building(&self) -> bool {
        self.building.as_ref().is_some_and(|building| {
            building.id > 0
                && (building.id != Buildings::TreeCouple
                    || matches!(self.terrain.slope, TerrainSlope::None | TerrainSlope::All))
        })
    }

    pub fn has_surface_water(&self) -> bool {
        matches!(
            self.terrain.ty,
            TerrainType::SurfaceWater | TerrainType::MoreSurfaceWater
        )
    }
}

impl TryFromDictionary for Tile {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            altitude: get_dict_key(value, "altitude")?,
            terrain: TileTerrainInfo::from(get_dict_key::<u32>(value, "terrain")?),
            coordinates: array_to_tuple(&get_dict_key(value, "coordinates")?)?,
            building: get_dict_key_optional(value, "building")?
                .map(|value| Building::try_from_dict(&value))
                .transpose()?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct SimulatorSettings {
    pub sea_level: u32,
}

impl TryFromDictionary for SimulatorSettings {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            sea_level: get_dict_key(value, "GlobalSeaLevel")?,
        })
    }
}

impl<T: TryFromDictionary> TryFromDictionary for BTreeMap<TileCoords, T> {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        value
            .iter_shared()
            .map(|(key, value)| {
                let key = key
                    .try_to()
                    .map_err(ErasedConvertError::from)
                    .map_err(TryFromDictError::InvalidKey)
                    .and_then(|val| array_to_tuple(&val))?;

                let value: Dictionary = value.try_to().map_err(|err| {
                    TryFromDictError::InvalidType(format!("{key:?}").into(), err.into())
                })?;

                Ok((key, T::try_from_dict(&value)?))
            })
            .collect()
    }
}

fn get_dict_key<T: FromGodot>(
    value: &Dictionary,
    key: &'static str,
) -> Result<T, TryFromDictError> {
    value
        .get(key)
        .ok_or(TryFromDictError::MissingKey(key))?
        .try_to()
        .map_err(|err| TryFromDictError::InvalidType(key.into(), err.into()))
}

fn get_dict_key_optional<T: FromGodot>(
    value: &Dictionary,
    key: &'static str,
) -> Result<Option<T>, TryFromDictError> {
    let variant = value.get_or_nil(key);

    if variant.is_nil() {
        return Ok(None);
    }

    variant
        .try_to()
        .map_err(|err| TryFromDictError::InvalidType(key.into(), err.into()))
        .map(Some)
}

fn array_to_tuple(value: &VariantArray) -> Result<TileCoords, TryFromDictError> {
    Ok((
        value
            .get(0)
            .ok_or(TryFromDictError::MissingKey("(x, _)"))?
            .try_to()
            .map_err(|err| TryFromDictError::InvalidType("(x, _)".into(), err.into()))?,
        value
            .get(1)
            .ok_or(TryFromDictError::MissingKey("(_, y)"))?
            .try_to()
            .map_err(|err| TryFromDictError::InvalidType("(x, _)".into(), err.into()))?,
    ))
}
