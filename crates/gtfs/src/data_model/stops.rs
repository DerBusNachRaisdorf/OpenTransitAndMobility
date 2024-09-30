use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::id::{HasId, Id};

use crate::database::WithPrimaryKey;

use super::{IdString, Latitude, Longitude, Timezone, Url};

/// Location Type.
/// See <https://gtfs.org/schedule/reference/#stopstxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum LocationType {
    /// **Stop** (or **Platform**). A location where passengers board or disembark
    /// from a transit vehicle. Is called a platform when defined within a
    /// `parent_station`.
    #[default]
    StopOrPlatform = 0,

    /// **Station**. A physical structure or area that contains one or more platform.
    Station = 1,

    /// **Entrance**/**Exit**. A location where passengers can enter or exit a station
    /// from the street. If an entrance/exit belongs to multiple stations, it may be
    /// linked by pathways to both, but the data provider must pick one of them as
    /// parent.
    EntranceExit = 2,

    /// **Generic Node**. A location within a station, not matching any other
    /// `location_type`, that may be used to link together pathways defined in
    /// pathways.txt.
    GenericNode = 3,

    /// **Boarding Area**. A specific location on a platform, where passengers can
    /// board and/or alight vehicles.
    BoardingArea = 4,
}

/// Indicates whether wheelchair boardings are possible from the location. Valid
/// options are:
///
/// For parentless stops:
/// 0 or empty - No accessibility information for the stop.
/// 1 - Some vehicles at this stop can be boarded by a rider in a wheelchair.
/// 2 - Wheelchair boarding is not possible at this stop.
///
/// For child stops:
/// 0 or empty - Stop will inherit its `wheelchair_boarding` behavior from the parent
///              station, if specified in the parent.
/// 1 - There exists some accessible path from outside the station to the specific
///     stop/platform.
/// 2 - There exists no accessible path from outside the station to the specific
///     stop/platform.
///
/// For station entrances/exits:
/// 0 or empty - Station entrance will inherit its `wheelchair_boarding` behavior from
///              the parent station, if specified for the parent.
/// 1 - Station entrance is wheelchair accessible.
/// 2 - No accessible path from station entrance to stops/platforms.
/// See <https://gtfs.org/schedule/reference/#stopstxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum WheechairBoarding {
    #[default]
    NoInformationOrInherit = 0,
    SomeAccessable = 1,
    NotAccessable = 2,
}

pub type StopId = Id<Stop>;

/// Stops where vehicles pick up or drop off riders. Also defines stations and
/// station entrances.
/// Primary Key: `stop_id`.
/// See <https://gtfs.org/schedule/reference/#stopstxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stop {
    /// Unique Primary Key.
    /// Identifies a location: stop/platform, station, entrance/exit, generic node or
    /// boarding area (see `location_type`).
    ///
    /// ID must be unique across all `stops.stop_id`, locations.geojson `id`, and
    /// `location_groups.location_group_id` values.
    ///
    /// Multiple routes may use the same `stop_id`.
    #[serde(rename = "stop_id")]
    pub id: Id<Stop>,

    /// Short text or a number that identifies the location for riders. These codes
    /// are often used in phone-based transit information systems or printed on
    /// signage to make it easier for riders to get information for a particular
    /// location. The stop_code may be the same as stop_id if it is public facing.
    /// This field should be left empty for locations without a code presented to
    /// riders.
    #[serde(rename = "stop_code")]
    pub code: Option<String>,

    /// Name of the location. The `stop_name` should match the agency's rider-facing
    /// name for the location as printed on a timetable, published online, or
    /// represented on signage. For translations into other languages, use
    /// translations.txt.
    ///
    /// When the location is a boarding area (`location_type=4`), the stop_name should
    /// contains the name of the boarding area as displayed by the agency. It could be
    /// just one letter (like on some European intercity railway stations), or text
    /// like “Wheelchair boarding area” (NYC’s Subway) or “Head of short trains”
    /// (Paris’ RER).
    ///
    /// Conditionally Required:
    /// - **Required** for locations which are stops (`location_type=0`), stations
    ///   (`location_type=1`) or entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes (`location_type=3`) or
    ///   boarding areas (`location_type=4`).
    #[serde(rename = "stop_name")]
    pub name: Option<String>,

    /// Readable version of the stop_name. See "Text-to-speech field" in the Term
    /// Definitions for more.
    #[serde(rename = "tts_stop_name")]
    pub tts_name: Option<String>,

    /// Description of the location that provides useful, quality information.
    /// Should not be a duplicate of `stop_name`.
    #[serde(rename = "stop_desc")]
    pub description: Option<String>,

    /// Latitude of the location.
    ///
    /// For stops/platforms (`location_type=0`) and boarding area (`location_type=4`),
    /// the coordinates must be the ones of the bus pole - if exists - and otherwise
    /// of where the travelers are boarding the vehicle (on the sidewalk or the
    /// platform, and not on the roadway or the track where the vehicle stops).
    ///
    /// Conditionally Required:
    /// - **Required** for locations which are stops (`location_type=0`), stations
    ///   (`location_type=1`) or entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes (`location_type=3`) or
    ///   boarding areas (location_type=4).
    #[serde(rename = "stop_lat")]
    pub latitude: Option<Latitude>,

    /// Longitude of the location.
    ///
    /// For stops/platforms (`location_type=0`) and boarding area (`location_type=4`),
    /// the coordinates must be the ones of the bus pole - if exists - and otherwise
    /// of where the travelers are boarding the vehicle (on the sidewalk or the
    /// platform, and not on the roadway or the track where the vehicle stops).
    ///
    /// Conditionally Required:
    /// - **Required** for locations which are stops (`location_type=0`), stations
    ///   (`location_type=1`) or entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes (`location_type=3`) or
    ///   boarding areas (`location_type=4`).
    #[serde(rename = "stop_lon")]
    pub longitude: Option<Longitude>,

    /// Identifies the fare zone for a stop. If this record represents a station or
    /// station entrance, the `zone_id` is ignored.
    pub zone_id: Option<IdString>, // TODO: use proper typed id?

    /// URL of a web page about the location. This should be different from the
    /// `agency.agency_url` and the `routes.route_url` field values.
    #[serde(rename = "stop_url")]
    pub url: Option<Url>,

    /// Location type.
    /// if empty: `LocationType::StopOrPlatform`
    #[serde(default)] // optional, but with default
    pub location_type: Option<LocationType>, // todo: this should not be an option...

    /// Foraign ID referencing `stops.stop_id`.
    ///
    /// Defines hierarchy between the different locations defined in stops.txt.
    /// It contains the ID of the parent location, as followed:
    ///
    /// - Stop/platform (`location_type=0`): the parent_station field contains the ID
    ///   of a station.
    /// - Station (`location_type=1`): this field must be empty.
    /// - Entrance/exit (`location_type=2`) or generic node (`location_type=3`): the
    ///   parent_station field contains the ID of a station (`location_type=1`)
    /// - Boarding Area (`location_type=4`): the parent_station field contains ID of a
    ///   platform.
    ///
    /// Conditionally Required:
    /// - **Required** for locations which are entrances (`location_type=2`), generic
    ///   nodes (`location_type=3`) or boarding areas (`location_type=4`).
    /// - Optional for stops/platforms (`location_type=0`).
    /// - Forbidden for stations (`location_type=1`).
    pub parent_station: Option<Id<Stop>>,

    /// Timezone of the location. If the location has a parent station, it inherits
    /// the parent station’s timezone instead of applying its own. Stations and
    /// parentless stops with empty `stop_timezone` inherit the timezone specified by
    /// `agency.agency_timezone`. The times provided in stop_times.txt are in the
    /// timezone specified by `agency.agency_timezone`, not `stop_timezone`. This ensures
    /// that the time values in a trip always increase over the course of a trip,
    /// regardless of which timezones the trip crosses.
    #[serde(rename = "stop_timezone")]
    pub timezone: Option<Timezone>,

    /// Indicates whether wheelchair boardings are possible from the location.
    #[serde(default)]
    pub wheelchair_boarding: WheechairBoarding,

    /// Foreign ID referencing `levels.level_id`
    ///
    /// Level of the location. The same level may be used by multiple unlinked
    /// stations.
    pub level_id: Option<IdString>, // TODO: use proper typed id.

    /// Platform identifier for a platform stop (a stop belonging to a station).
    /// This should be just the platform identifier (eg. "G" or "3"). Words like
    /// “platform” or "track" (or the feed’s language-specific equivalent) should not
    /// be included. This allows feed consumers to more easily internationalize and
    /// localize the platform identifier into other languages.
    pub platform_code: Option<String>,
}

impl HasId for Stop {
    type IdType = String;
}

impl Stop {
    pub fn wheelchair_boarding_text(self) -> String {
        todo!()
    }
}

impl WithPrimaryKey<StopId> for Stop {
    fn primary_key(&self) -> StopId {
        self.id.clone()
    }
}
