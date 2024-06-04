use std::collections::BTreeMap;

use godot::builtin::{
    meta::{ConvertError, FromGodot},
    Dictionary, VariantArray,
};

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

pub type TileList = BTreeMap<(u32, u32), Tile>;

#[derive(Debug)]
pub(crate) struct City {
    pub simulator_settings: SimulatorSettings,
    pub buildings: BTreeMap<(u32, u32), Building>,
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

#[derive(Debug)]
pub(crate) struct Building {
    pub size: u8,
    pub name: String,
    pub building_id: u8,
    pub tile_coords: (u32, u32),
}

impl TryFromDictionary for Building {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            size: get_dict_key(value, "size")?,
            name: get_dict_key(value, "name")?,
            building_id: get_dict_key(value, "building_id")?,
            tile_coords: get_dict_key(value, "tile_coords").and_then(array_to_tuple)?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Tile {
    pub altitude: u32,
    pub building: Option<Building>,
}

impl TryFromDictionary for Tile {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        Ok(Self {
            altitude: get_dict_key(value, "altitude")?,
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

impl<T: TryFromDictionary> TryFromDictionary for BTreeMap<(u32, u32), T> {
    fn try_from_dict(value: &Dictionary) -> Result<Self, TryFromDictError> {
        value
            .iter_shared()
            .map(|(key, value)| {
                let key = key
                    .try_to()
                    .map_err(ErasedConvertError::from)
                    .map_err(TryFromDictError::InvalidKey)
                    .and_then(array_to_tuple)?;

                let value: Dictionary = value.try_to().map_err(|err| {
                    TryFromDictError::InvalidType(format!("{:?}", key).into(), err.into())
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

fn array_to_tuple(value: VariantArray) -> Result<(u32, u32), TryFromDictError> {
    Ok((
        value
            .try_get(0)
            .ok_or(TryFromDictError::MissingKey("(x, _)"))?
            .try_to()
            .map_err(|err| TryFromDictError::InvalidType("(x, _)".into(), err.into()))?,
        value
            .try_get(1)
            .ok_or(TryFromDictError::MissingKey("(_, y)"))?
            .try_to()
            .map_err(|err| TryFromDictError::InvalidType("(x, _)".into(), err.into()))?,
    ))
}
