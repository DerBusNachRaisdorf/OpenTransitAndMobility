use std::fs;

use serde::{Deserialize, Serialize};
use serde_json;
use serde_with;

use crate::ApiError;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimetableOffice {
    /// email
    pub email: Option<String>,

    /// identifier
    pub name: String,
}

/// local public sector entity, responsible for short distance
/// public transport in a specific area
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aufgabentraeger {
    /// full name of Aufgabentraeger
    pub name: String,

    /// unique identifier
    pub short_name: String,
}

/// period of time from/to
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHours {
    /// example: 9:00
    /// pattern: ([0-9]|0[0-9]|1[0-9]|2[0-3]):[0-5][0-9]
    pub from_time: String,

    /// example: 23:00
    /// pattern: ^([0-9]|0[0-9]|1[0-9]|2[0-3]):[0-5][0-9]
    pub to_time: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub monday: OpeningHours,
    pub tuesday: OpeningHours,
    pub wednesday: OpeningHours,
    pub thursday: OpeningHours,
    pub friday: OpeningHours,
    pub saturday: OpeningHours,
    pub sunday: OpeningHours,
    pub holiday: OpeningHours,
}

/// a weekly schedule
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub availability: Availability,
}

/// GEOJSON object of type point.
/// By default, WGS84 is the coordinate system in GEOJSON.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeographicPoint {
    /// first value is longitude, second latitude, third altitude (currently not provided).
    /// TODO: own datatype
    pub coordinates: Vec<f64>,

    /// the type of the GEOJSON Object e.g. point.
    /// Currently only point coordinates without altitude are provided.
    #[serde(rename = "type")]
    pub geojson_type: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EVANumber {
    pub geographic_coordinates: Option<GeographicPoint>,

    /// isMain is supported for compatibility reasons only.
    /// The attribute has no business in terms of Station&Service AG.
    pub is_main: bool,

    /// EVA identifier.
    /// TODO: only integer is specified as type. Check whether it is actually a 64 bit integer.
    pub number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Partial {
    Yes,
    No,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub city: String,
    pub house_number: Option<String>,
    pub street: String,
    pub zipcode: String,
}

/// reference object. an internal organization type of Station&Sevice, regional department.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionalBereich {
    /// name of the regional department
    pub name: String,

    /// unique identifier of the regional department
    pub number: i32,

    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SteamPermission {
    /// Eingeschränkt
    Restricted,

    /// Uneingeschränkt
    Unrestricted,

    /// Einfahrverbot
    EntryBan,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiL100Identifier {
    pub geographic_coordinates: Option<GeographicPoint>,

    // permissions for steam engines y/n
    // depricated
    //pub has_steam_permissions: bool,
    /// is stations main Ril100.
    /// Determination of Station&Service AG
    pub is_main: bool,

    /// UIC Primary Location Code PLC
    pub primary_location_code: String,

    /// unique identifier of 'Betriebsstelle' according to Ril100
    pub ril_identifier: String,

    /// Indicates whether the entry for a steam engine is restricted (eingeschränkt),
    /// unrestricted (uneingeschränkt) or has an entryBan (Einfahrverbot).
    pub steam_permission: SteamPermission,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationManagement {
    pub name: String,

    /// identifier
    /// TODO: integer type not specified, check whether i64 is suited.
    pub number: i64,
}

/// 3-S-Zentralen are 7/24 hours operating centers for german railway stations
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SZentrale {
    pub address: Option<Address>,

    // email adress of the 3-S-Zentrale (no longer supported!)
    //pub email: String,
    /// internal fax number
    pub internal_fax_number: Option<String>,

    /// internal phone number
    pub internal_phone_number: Option<String>,

    // mobile phone number (no longer supported!)
    //pub mobile_phone_number: String,
    /// unique identifier of 3SZentrale
    pub name: String,

    /// unique identifier for SZentrale
    pub number: i32,

    /// public fax number
    pub public_fax_number: Option<String>,

    pub public_phone_number: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SZentraleQuery {
    /// maximum number of result objects to be returned
    pub limit: i64,

    /// offset of the first result object with respect to the total number of hits produced by the
    /// query
    pub offset: i64,

    /// result objects produced by that query
    pub result: Vec<SZentrale>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLine {
    pub product_line: String,
    pub segment: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    #[serde(rename = "DBinformation")]
    pub db_information: Option<Schedule>,

    /// the stations category (-1...7).
    /// Stations with category -1 or 0 are not in production,
    /// e.g. planned, saled or without train stops.
    pub category: i32,

    /// station related EVA-numbers
    pub eva_numbers: Vec<EVANumber>,

    /// german federal state
    pub federal_state: String,

    /// public bicycle parking y/n
    pub has_bicycle_parking: bool,

    /// car sharing or car rental y/n
    pub has_car_rental: bool,

    /// DB lounge y/n
    #[serde(rename = "hasDBLounge")]
    pub has_db_lounge: bool,

    /// local public transport y/n
    pub has_local_public_transport: bool,

    /// public facilities y/n
    pub has_locker_system: bool,

    /// lost and found y/n
    pub has_lost_and_found: bool,

    /// values are 'no', 'yes, advance notification is requested...' or 'yes, advance notification is required...'
    pub has_mobility_service: String,

    /// public parking y/n
    pub has_parking: bool,

    /// public facilities y/n
    pub has_public_facilities: bool,

    /// railway mission y/n
    pub has_railway_mission: bool,

    pub has_stepless_access: Partial,

    /// taxi rank in front of the station y/n
    pub has_taxi_rank: bool,

    /// local travel center y/n
    pub has_travel_center: bool,

    /// a shop for travel necessities y/n
    pub has_travel_necessities: bool,

    /// public Wi-Fi is available y/n
    #[serde(rename = "hasWiFi")]
    pub has_wifi: bool,

    pub local_service_staff: Option<Schedule>,

    pub mailing_address: Address,

    /// the stations name
    pub name: String,

    /// unique identifier representing a specific railway station
    pub number: i32,

    /// determines in some respect the price for train stops at a specific station (1..7)
    pub price_category: i32,

    pub regionalbereich: RegionalBereich,

    /// station related Ril100s
    pub ril100_identifiers: Vec<RiL100Identifier>,

    pub station_management: StationManagement,

    pub szentrale: SZentrale,

    #[serde(alias = "timeTableOffice")]
    pub timetable_office: TimetableOffice,

    pub product_line: Option<ProductLine>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationQuery {
    /// maximum number of result objects to be returned
    pub limit: i64,

    /// offset of the first result object with respect to the total number
    /// of hits produced by the query
    pub offset: i64,

    /// result objects produced by the query
    pub result: Vec<Station>,

    /// total number of hits produced by that query
    pub total: i64,
}

impl StationQuery {
    pub fn load_from_file(path: &str) -> Result<Self, ApiError> {
        // TODO: proper error handling
        let file_content = fs::read_to_string(path)
            .map_err(|why| ApiError::Other(why.to_string()))?;
        serde_json::from_str(&file_content)
            .map_err(|why| ApiError::Other(why.to_string()))
    }
}
