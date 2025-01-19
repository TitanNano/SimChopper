#![allow(dead_code)]

use godot::classes::{PackedScene, ResourceLoader};
use godot::global::godot_warn;
use godot::obj::Gd;
use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum Powerlines {
    LeftRight = 0x0E,
    TopBottom = 0x0F,
    HighTopBottom = 0x10,
    LeftHighRight = 0x11,
    TopHighBottom = 0x12,
    HighLeftRight = 0x13,
    BottomRight = 0x14,
    BottomLeft = 0x15,
    TopLeft = 0x16,
    TopRight = 0x17,
    RightTopBottom = 0x18,
    LeftBottomRight = 0x19,
    TopLeftBottom = 0x1A,
    LeftTopRight = 0x1B,
    LeftTopBottomRight = 0x1C,
    BridgeTopBottom = 0x5C,
}

impl Powerlines {
    fn as_str(&self) -> &'static str {
        match self {
            Powerlines::LeftRight => "res://resources/Objects/Networks/Powerline/left_right.tscn",
            Powerlines::TopBottom => "res://resources/Objects/Networks/Powerline/top_bottom.tscn",
            Powerlines::HighTopBottom => {
                "res://resources/Objects/Networks/Powerline/high_top_bottom.tscn"
            }
            Powerlines::LeftHighRight => {
                "res://resources/Objects/Networks/Powerline/left_high_right.tscn"
            }
            Powerlines::TopHighBottom => {
                "res://resources/Objects/Networks/Powerline/top_high_bottom.tscn"
            }
            Powerlines::HighLeftRight => {
                "res://resources/Objects/Networks/Powerline/high_left_right.tscn"
            }
            Powerlines::BottomRight => {
                "res://resources/Objects/Networks/Powerline/bottom_right.tscn"
            }
            Powerlines::BottomLeft => "res://resources/Objects/Networks/Powerline/bottom_left.tscn",
            Powerlines::TopLeft => "res://resources/Objects/Networks/Powerline/top_left.tscn",
            Powerlines::TopRight => "res://resources/Objects/Networks/Powerline/top_right.tscn",
            Powerlines::RightTopBottom => {
                "res://resources/Objects/Networks/Powerline/right_top_bottom.tscn"
            }
            Powerlines::LeftBottomRight => {
                "res://resources/Objects/Networks/Powerline/left_bottom_right.tscn"
            }
            Powerlines::TopLeftBottom => {
                "res://resources/Objects/Networks/Powerline/top_left_bottom.tscn"
            }
            Powerlines::LeftTopRight => {
                "res://resources/Objects/Networks/Powerline/left_top_right.tscn"
            }
            Powerlines::LeftTopBottomRight => {
                "res://resources/Objects/Networks/Powerline/left_top_bottom_right.tscn"
            }
            Powerlines::BridgeTopBottom => {
                "res://resources/Objects/Networks/Powerline/bridge_top_bottom.tscn"
            }
        }
    }
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum Road {
    LeftRight = 0x1D,
    TopBottom = 0x1E,
    HighTopBottom = 0x1F,
    LeftHighRight = 0x20,
    TopHighBottom = 0x21,
    HighLeftRight = 0x22,
    TopRight = 0x23,
    BottomRight = 0x24,
    BottomLeft = 0x25,
    TopLeft = 0x26,
    RightTopBottom = 0x27,
    LeftBottomRight = 0x28,
    TopLeftBottom = 0x29,
    LeftTopRight = 0x2A,
    LeftTopBottomRight = 0x2B,
    LeftRightPowerTopBottom = 0x43,
    TopBottomPowerLeftRight = 0x44,
}

impl Road {
    fn as_str(&self) -> &'static str {
        match self {
            Self::LeftRight => "res://resources/Objects/Networks/Road/left_right.tscn",
            Self::TopBottom => "res://resources/Objects/Networks/Road/top_bottom.tscn",
            Self::HighTopBottom => "res://resources/Objects/Networks/Road/high_top_bottom.tscn",
            Self::LeftHighRight => "res://resources/Objects/Networks/Road/left_high_right.tscn",
            Self::TopHighBottom => "res://resources/Objects/Networks/Road/top_high_bottom.tscn",
            Self::HighLeftRight => "res://resources/Objects/Networks/Road/high_left_right.tscn",
            Self::TopRight => "res://resources/Objects/Networks/Road/top_right.tscn",
            Self::BottomRight => "res://resources/Objects/Networks/Road/bottom_right.tscn",
            Self::BottomLeft => "res://resources/Objects/Networks/Road/bottom_left.tscn",
            Self::TopLeft => "res://resources/Objects/Networks/Road/top_left.tscn",
            Self::RightTopBottom => "res://resources/Objects/Networks/Road/right_top_bottom.tscn",
            Self::LeftBottomRight => "res://resources/Objects/Networks/Road/left_bottom_right.tscn",
            Self::TopLeftBottom => "res://resources/Objects/Networks/Road/top_left_bottom.tscn",
            Self::LeftTopRight => "res://resources/Objects/Networks/Road/left_top_right.tscn",
            Self::LeftTopBottomRight => {
                "res://resources/Objects/Networks/Road/left_top_bottom_right.tscn"
            }
            Self::LeftRightPowerTopBottom => {
                "res://resources/Objects/Networks/Road/left_right_power_top_bottom.tscn"
            }
            Self::TopBottomPowerLeftRight => {
                "res://resources/Objects/Networks/Road/top_bottom_power_left_right.tscn"
            }
        }
    }
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum SuspensionBridge {
    StartBottom = 0x51,
    MiddleBottom = 0x52,
    Center = 0x53,
    MiddleTop = 0x54,
    EndTop = 0x55,
}

impl SuspensionBridge {
    fn as_str(&self) -> &'static str {
        match self {
            Self::StartBottom => {
                "res://resources/Objects/Networks/Bridge/bridge_suspension_start_bottom.tscn"
            }
            Self::MiddleBottom => {
                "res://resources/Objects/Networks/Bridge/bridge_suspension_middle_bottom.tscn"
            }
            Self::Center => "res://resources/Objects/Networks/Bridge/bridge_suspension_center.tscn",
            Self::MiddleTop => {
                "res://resources/Objects/Networks/Bridge/bridge_suspension_middle_top.tscn"
            }
            Self::EndTop => {
                "res://resources/Objects/Networks/Bridge/bridge_suspension_end_top.tscn"
            }
        }
    }
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum PylonBridge {
    RaisingTowerTopBottom = 0x56,
    BridgeTopA = 0x57,
    BridgeTopB = 0x58,
}

impl PylonBridge {
    fn as_str(&self) -> &'static str {
        match self {
            Self::RaisingTowerTopBottom => {
                "res://resources/Objects/Networks/Bridge/bridge_raising_tower_top_bottom.tscn"
            }
            Self::BridgeTopA => "res://resources/Objects/Networks/Bridge/bridge_top.tscn",
            Self::BridgeTopB => "res://resources/Objects/Networks/Bridge/bridge_top.tscn",
        }
    }
}

fn networks(id: u8) -> Option<&'static str> {
    // Powerlines
    Powerlines::try_from_primitive(id)
        .ok()
        .as_ref()
        .map(Powerlines::as_str)
        .or_else(|| Road::try_from_primitive(id).ok().as_ref().map(Road::as_str))
        .or_else(|| {
            SuspensionBridge::try_from_primitive(id)
                .ok()
                .as_ref()
                .map(SuspensionBridge::as_str)
        })
        .or_else(|| {
            PylonBridge::try_from_primitive(id)
                .ok()
                .as_ref()
                .map(PylonBridge::as_str)
        })
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Buildings {
    ParkSmall = 0x0D,
    TreeSingle = 0x06,
    HomeMiddleClass1 = 0x73,
    HomeMiddleClass2 = 0x74,
    HomeMiddleClass3 = 0x75,
    HomeMiddleClass4 = 0x76,
    HomeMiddleClass5 = 0x77,
    Church = 0xF7,
    OfficeBuildingMedium1 = 0x96,
    OfficeBuildingMedium2 = 0x98,
    OfficeBuildingMedium3 = 0x9A,
    OfficeBuildingMedium4 = 0x9B,
    OfficeBuildingMedium5 = 0x9C,
    OfficeBuildingMedium6 = 0x9D,
    AbandonedBuilding1 = 0x8A,
    AbandonedBuilding2 = 0x8B,
    AbandonedBuilding3 = 0xAA,
    AbandonedBuilding4 = 0xAB,
    AbandonedBuilding5 = 0xAC,
    AbandonedBuilding6 = 0xAD,
    HomeUpperClass1 = 0x78,
    HomeUpperClass2 = 0x79,
    HomeUpperClass3 = 0x7A,
    HomeUpperClass4 = 0x7B,
    Tarmac = 0xE6,
    TarmacRadar = 0xEA,
    Construction1 = 0xC2,
    Construction2 = 0xC3,
    Construction3 = 0xA6,
    Construction4 = 0xA7,
    Construction5 = 0xA8,
    Construction6 = 0xA9,
    Construction7 = 0x88,
    Construction8 = 0x89,
    AirportWarehouse = 0xE3,
    AirportBuilding1 = 0xE4,
    AirportBuilding2 = 0xE5,
    AirportHangar1 = 0xE8,
    AirportRunway = 0xDD,
    AirportRunwayIntersection = 0xDF,
    Hangar2 = 0xF6,
    CondominiumsMedium1 = 0x91,
    CondominiumsMedium2 = 0x92,
    CondominiumsMedium3 = 0x93,
    CondominiumsLarge1 = 0xB0,
    CondominiumsLarge2 = 0xB1,
    FactorySmall1 = 0xA0,
    FactorySmall2 = 0xA1,
    FactorySmall3 = 0xA2,
    FactorySmall4 = 0xA3,
    FactorySmall5 = 0xA4,
    FactorySmall6 = 0xA5,
    StationPolice = 0xD2,
    ApartmentsMedium1 = 0x8F,
    ApartmentsMedium2 = 0x90,
    ToyStore = 0x83,
    IndustrialSubstation = 0x87,
    OfficesSmall1 = 0x80,
    OfficesSmall2 = 0x81,
    OfficesHistoric = 0xBA,
    WaterPump = 0xDC,
    StationHospital = 0xD1,
    ConvenienceStore = 0x7E,
    StationGas1 = 0x7C,
    StationGas2 = 0x7F,
    HomeLowerClass1 = 0x70,
    HomeLowerClass2 = 0x71,
    HomeLowerClass3 = 0x72,
    Warehouse = 0x82,
    AirportCivilianControlTower = 0xE1,
    StationFire = 0xD3,
    PowerplantMicrowave = 0xCD,
    ResortHotel = 0x97,
    ApartmentsLarge1 = 0xAE,
    ApartmentsLarge2 = 0xAF,
    ApartmentsSmall1 = 0x8C,
    ApartmentsSmall2 = 0x8D,
    ApartmentsSmall3 = 0x8E,
    TreeCouple = 0x07,
    ChemicalStorage = 0x85,
    ChemicalProcessing1 = 0xBC,
    ChemicalProcessing2 = 0x9F,
    School = 0xD6,
    Library = 0xF5,
    Marina = 0xF8,
    WarehouseLarge1 = 0xC0,
    WarehouseLarge2 = 0xC1,
    WarehouseSmall1 = 0x84,
    WarehouseSmall2 = 0x86,
    WarehouseMedium = 0x9E,
    BbInn = 0x7D,
    College = 0xD9,
    ArcologyPlymouth = 0xFB,
    ArcologyForest = 0xFC,
    ArcologyDarco = 0xFD,
    ArcologyLaunch = 0xFE,
    MayorsHouse = 0xF3,
    Museum = 0xD4,
    OfficeRetail = 0x99,
    ParkingLot = 0xB9,
    ShoppingCentre = 0x94,
    Theatre = 0xB5,
    WaterTreatment = 0xF4,
}

impl Buildings {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ParkSmall => "res://resources/Objects/Buildings/park_small.tscn",
            Self::TreeSingle => "res://resources/Objects/Buildings/tree_single.tscn",
            Self::HomeMiddleClass1 => "res://resources/Objects/Buildings/home_middle_class_1.tscn",
            Self::HomeMiddleClass2 => "res://resources/Objects/Buildings/home_middle_class_2.tscn",
            Self::HomeMiddleClass3 => "res://resources/Objects/Buildings/home_middle_class_3.tscn",
            Self::HomeMiddleClass4 => "res://resources/Objects/Buildings/home_middle_class_4.tscn",
            Self::HomeMiddleClass5 => "res://resources/Objects/Buildings/home_middle_class_5.tscn",
            Self::Church => "res://resources/Objects/Buildings/church.tscn",
            Self::OfficeBuildingMedium1 => {
                "res://resources/Objects/Buildings/office_building_medium_1.tscn"
            }
            Self::OfficeBuildingMedium2 => {
                "res://resources/Objects/Buildings/office_building_medium_2.tscn"
            }
            Self::OfficeBuildingMedium3 => {
                "res://resources/Objects/Buildings/office_building_medium_3.tscn"
            }
            Self::OfficeBuildingMedium4 => {
                "res://resources/Objects/Buildings/office_building_medium_4.tscn"
            }
            Self::OfficeBuildingMedium5 => {
                "res://resources/Objects/Buildings/office_building_medium_5.tscn"
            }
            Self::OfficeBuildingMedium6 => {
                "res://resources/Objects/Buildings/office_building_medium_6.tscn"
            }
            Self::AbandonedBuilding1 => {
                "res://resources/Objects/Buildings/abandoned_building_1.tscn"
            }
            Self::AbandonedBuilding2 => {
                "res://resources/Objects/Buildings/abandoned_building_2.tscn"
            }
            Self::AbandonedBuilding3 => {
                "res://resources/Objects/Buildings/abandoned_building_3.tscn"
            }
            Self::AbandonedBuilding4 => {
                "res://resources/Objects/Buildings/abandoned_building_4.tscn"
            }
            Self::AbandonedBuilding5 => {
                "res://resources/Objects/Buildings/abandoned_building_5.tscn"
            }
            Self::AbandonedBuilding6 => {
                "res://resources/Objects/Buildings/abandoned_building_6.tscn"
            }
            Self::HomeUpperClass1 => "res://resources/Objects/Buildings/home_upper_class_1.tscn",
            Self::HomeUpperClass2 => "res://resources/Objects/Buildings/home_upper_class_2.tscn",
            Self::HomeUpperClass3 => "res://resources/Objects/Buildings/home_upper_class_3.tscn",
            Self::HomeUpperClass4 => "res://resources/Objects/Buildings/home_upper_class_4.tscn",
            Self::Tarmac => "res://resources/Objects/Ground/tarmac.tscn",
            Self::TarmacRadar => "res://resources/Objects/Buildings/tarmac_radar.tscn",
            Self::Construction1 => "res://resources/Objects/Buildings/construction_1-2.tscn",
            Self::Construction2 => "res://resources/Objects/Buildings/construction_1-2.tscn",
            Self::Construction3 => "res://resources/Objects/Buildings/construction_3.tscn",
            Self::Construction4 => "res://resources/Objects/Buildings/construction_4.tscn",
            Self::Construction5 => "res://resources/Objects/Buildings/construction_5.tscn",
            Self::Construction6 => "res://resources/Objects/Buildings/construction_6.tscn",
            Self::Construction7 => "res://resources/Objects/Buildings/construction_7.tscn",
            Self::Construction8 => "res://resources/Objects/Buildings/construction_8.tscn",
            Self::AirportWarehouse => "res://resources/Objects/Buildings/airport_warehouse.tscn",
            Self::AirportBuilding1 => "res://resources/Objects/Buildings/airport_building_1.tscn",
            Self::AirportBuilding2 => "res://resources/Objects/Buildings/airport_building_2.tscn",
            Self::AirportHangar1 => "res://resources/Objects/Buildings/airport_hangar_1.tscn",
            Self::AirportRunway => "res://resources/Objects/Buildings/airport_runway.tscn",
            Self::AirportRunwayIntersection => {
                "res://resources/Objects/Buildings/airport_runway_intersection.tscn"
            }
            Self::Hangar2 => "res://resources/Objects/Buildings/hangar_2.tscn",
            Self::CondominiumsMedium1 => {
                "res://resources/Objects/Buildings/condominiums_medium_1.tscn"
            }
            Self::CondominiumsMedium2 => {
                "res://resources/Objects/Buildings/condominiums_medium_2.tscn"
            }
            Self::CondominiumsMedium3 => {
                "res://resources/Objects/Buildings/condominiums_medium_3.tscn"
            }
            Self::CondominiumsLarge1 => {
                "res://resources/Objects/Buildings/condominiums_large_1.tscn"
            }
            Self::CondominiumsLarge2 => {
                "res://resources/Objects/Buildings/condominiums_large_2.tscn"
            }
            Self::FactorySmall1 => "res://resources/Objects/Buildings/factory_small_1.tscn",
            Self::FactorySmall2 => "res://resources/Objects/Buildings/factory_small_2.tscn",
            Self::FactorySmall3 => "res://resources/Objects/Buildings/factory_small_3.tscn",
            Self::FactorySmall4 => "res://resources/Objects/Buildings/factory_small_4.tscn",
            Self::FactorySmall5 => "res://resources/Objects/Buildings/factory_small_5.tscn",
            Self::FactorySmall6 => "res://resources/Objects/Buildings/factory_small_6.tscn",
            Self::StationPolice => "res://resources/Objects/Buildings/station_police.tscn",
            Self::ApartmentsMedium1 => "res://resources/Objects/Buildings/apartments_medium_1.tscn",
            Self::ApartmentsMedium2 => "res://resources/Objects/Buildings/apartments_medium_2.tscn",
            Self::ToyStore => "res://resources/Objects/Buildings/toy_store.tscn",
            Self::IndustrialSubstation => {
                "res://resources/Objects/Buildings/industrial_substation.tscn"
            }
            Self::OfficesSmall1 => "res://resources/Objects/Buildings/offices_small_1.tscn",
            Self::OfficesSmall2 => "res://resources/Objects/Buildings/offices_small_2.tscn",
            Self::OfficesHistoric => "res://resources/Objects/Buildings/offices_historic.tscn",
            Self::WaterPump => "res://resources/Objects/Buildings/water_pump.tscn",
            Self::StationHospital => "res://resources/Objects/Buildings/station_hospital.tscn",
            Self::ConvenienceStore => "res://resources/Objects/Buildings/convenience_store.tscn",
            Self::StationGas1 => "res://resources/Objects/Buildings/station_gas_1.tscn",
            Self::StationGas2 => "res://resources/Objects/Buildings/station_gas_2.tscn",
            Self::HomeLowerClass1 => "res://resources/Objects/Buildings/home_lower_class_1.tscn",
            Self::HomeLowerClass2 => "res://resources/Objects/Buildings/home_lower_class_2.tscn",
            Self::HomeLowerClass3 => "res://resources/Objects/Buildings/home_lower_class_3.tscn",
            Self::Warehouse => "res://resources/Objects/Buildings/warehouse.tscn",
            Self::AirportCivilianControlTower => {
                "res://resources/Objects/Buildings/airport_civilian_control_tower.tscn"
            }
            Self::StationFire => "res://resources/Objects/Buildings/station_fire.tscn",
            Self::PowerplantMicrowave => {
                "res://resources/Objects/Buildings/powerplant_microwave.tscn"
            }
            Self::ResortHotel => "res://resources/Objects/Buildings/resort_hotel.tscn",
            Self::ApartmentsLarge1 => "res://resources/Objects/Buildings/apartments_large_1.tscn",
            Self::ApartmentsLarge2 => "res://resources/Objects/Buildings/apartments_large_2.tscn",
            Self::ApartmentsSmall1 => "res://resources/Objects/Buildings/apartments_small_1.tscn",
            Self::ApartmentsSmall2 => "res://resources/Objects/Buildings/apartments_small_2.tscn",
            Self::ApartmentsSmall3 => "res://resources/Objects/Buildings/apartments_small_3.tscn",
            Self::TreeCouple => "res://resources/Objects/Buildings/tree_couple.tscn",
            Self::ChemicalStorage => "res://resources/Objects/Buildings/chemical_storage.tscn",
            Self::ChemicalProcessing1 => {
                "res://resources/Objects/Buildings/chemical_processing_1.tscn"
            }
            Self::ChemicalProcessing2 => {
                "res://resources/Objects/Buildings/chemical_processing_2.tscn"
            }
            Self::School => "res://resources/Objects/Buildings/school.tscn",
            Self::Library => "res://resources/Objects/Buildings/library.tscn",
            Self::Marina => "res://resources/Objects/Buildings/marina.tscn",
            Self::WarehouseLarge1 => "res://resources/Objects/Buildings/warehouse_large_1.tscn",
            Self::WarehouseLarge2 => "res://resources/Objects/Buildings/warehouse_large_2.tscn",
            Self::WarehouseSmall1 => "res://resources/Objects/Buildings/warehouse_small_1.tscn",
            Self::WarehouseSmall2 => "res://resources/Objects/Buildings/warehouse_small_2.tscn",
            Self::WarehouseMedium => "res://resources/Objects/Buildings/warehouse_medium.tscn",
            Self::BbInn => "res://resources/Objects/Buildings/bb_inn.tscn",
            Self::College => "res://resources/Objects/Buildings/college.tscn",
            Self::ArcologyPlymouth => "res://resources/Objects/Buildings/arcology_plymouth.tscn",
            Self::ArcologyForest => "res://resources/Objects/Buildings/arcology_forest.tscn",
            Self::ArcologyDarco => "res://resources/Objects/Buildings/arcology_darco.tscn",
            Self::ArcologyLaunch => "res://resources/Objects/Buildings/arcology_launch.tscn",
            Self::MayorsHouse => "res://resources/Objects/Buildings/mayors_house.tscn",
            Self::Museum => "res://resources/Objects/Buildings/museum.tscn",
            Self::OfficeRetail => "res://resources/Objects/Buildings/office_retail.tscn",
            Self::ParkingLot => "res://resources/Objects/Buildings/parking_lot.tscn",
            Self::ShoppingCentre => "res://resources/Objects/Buildings/shopping_centre.tscn",
            Self::Theatre => "res://resources/Objects/Buildings/theatre.tscn",
            Self::WaterTreatment => "res://resources/Objects/Buildings/water_treatment.tscn",
        }
    }
}

impl PartialEq<u8> for Buildings {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}

impl PartialEq<Buildings> for u8 {
    fn eq(&self, other: &Buildings) -> bool {
        other == self
    }
}

fn buildings(id: u8) -> Option<&'static str> {
    Buildings::try_from_primitive(id)
        .ok()
        .as_ref()
        .map(Buildings::as_str)
}

fn load(parser: fn(u8) -> Option<&'static str>, object_id: u8) -> Option<Gd<PackedScene>> {
    let Some(object) = parser(object_id) else {
        godot_warn!("{:02x} is not a valid object id", object_id);
        return None;
    };

    ResourceLoader::singleton().load(object).map(Gd::cast)
}

pub fn load_network(object_id: u8) -> Option<Gd<PackedScene>> {
    load(networks, object_id)
}

pub fn load_building(object_id: u8) -> Option<Gd<PackedScene>> {
    load(buildings, object_id)
}
