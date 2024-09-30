use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::id::{HasId, Id};

use crate::database::WithPrimaryKey;

use super::{agency::Agency, IdString, Url};

/// Indicates the type of transportation used on a route.
/// See <https://gtfs.org/schedule/reference/#routestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum RouteType {
    /// Tram, Streetcar, Light rail.
    /// Any light rail or street level system within a metropolitan area.
    TramStreetcarOrLighrail = 0,

    /// Subway, Metro. Any underground rail system within a metropolitan area.
    SubwayOrMetro = 1,

    /// Rail. Used for intercity or long-distance travel.
    Rail = 2,

    /// Bus. Used for short- and long-distance bus routes.
    Bus = 3,

    /// Ferry. Used for short- and long-distance boat service.
    Ferry = 4,

    /// Cable tram. Used for street-level rail cars where the cable runs beneath the
    /// vehicle (e.g., cable car in San Francisco).
    CableTram = 5,

    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway).
    /// Cable transport where cabins, cars, gondolas or open chairs are suspended by
    /// means of one or more cables.
    AerialLiftOrSuspendedCableCar = 6,

    /// Funicular. Any rail system designed for steep inclines.
    Funicular = 7,

    /// Trolleybus. Electric buses that draw power from overhead wires using poles.
    Trolleybus = 11,

    /// Monorail. Railway in which the track consists of a single rail or a beam.
    Monorail = 12,
}

impl RouteType {
    pub fn display_text(self) -> String {
        match self {
            Self::TramStreetcarOrLighrail => {
                "Tram, Streetcar or Light rail. \
                 A light rail or street level system within a metropolitan area."
            }
            Self::SubwayOrMetro => {
                "Subway or Metro. \
                 A underground rail system within a metropolitan area."
            }
            Self::Rail => "Rail. An intercity or long distance travel.",
            Self::Bus => "Bus. A short- or long-distance bus route.",
            Self::Ferry => "Ferry. A short- or long-distance boat service.",
            Self::CableTram => {
                "Cable tram. \
                 A street-level rail car where the cable runs beneath the vehicle."
            }
            Self::AerialLiftOrSuspendedCableCar => {
                "Aerial lift, suspended cable car \
                 (e.g., gondola lift, aerial tramway). \
                 Cable transport where cabins, cars, gondolas or open chairs are \
                 suspended by means of on eore more cables."
            }
            Self::Funicular => {
                "Funicular. \
                 A rail system designed for steep inclines."
            }
            Self::Trolleybus => {
                "Trolleybus. \
                 An electric bus that draws power from overhead wires using poles."
            }
            Self::Monorail => {
                "Monorail. \
                 A Railway in which the track consists of a single rail or beam."
            }
        }
        .to_owned()
    }
}

/// Indicates that the rider can board the transit vehicle at any point along the
/// vehicle’s travel path as described by shapes.txt, on every trip of the route.
/// See <https://gtfs.org/schedule/reference/#routestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum ContinuousPickup {
    /// Continuous stopping pickup.
    Yes = 0,

    /// No continuous stopping pickup.
    #[default]
    No = 1,

    /// Must phone agency to arrange continuous stopping pickup.
    MustPhoneAgency = 2,

    /// Must coordinate with driver to arrange continuous stopping pickup.
    MustCoordinateWithDriver = 3,
}

impl ContinuousPickup {
    pub fn display_text(self) -> String {
        match self {
            Self::Yes => "Continuous stopping pickup.",
            Self::No => "No continuous stopping pickup.",
            Self::MustPhoneAgency => {
                "Must phone agency to arrange continuous stopping pickup. "
            }
            Self::MustCoordinateWithDriver => {
                "Must coordinate with driver to arrange continuous stopping pickup. "
            }
        }
        .to_owned()
    }
}

/// Indicates that the rider can alight from the transit vehicle at any point along
/// the vehicle’s travel path as described by shapes.txt, on every trip of the route.
/// See <https://gtfs.org/schedule/reference/#routestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum ContinuousDropOff {
    /// Continuous stopping drop off.
    Yes = 0,

    /// No continuous stopping drop off.
    #[default]
    No = 1,

    /// Must phone agency to arrange continuous stopping drop off.
    MustPhoneAgency = 2,

    /// Must coordinate with driver to arrange continuous stopping drop off.
    MustCoordinateWithDriver = 3,
}

impl ContinuousDropOff {
    pub fn display_text(self) -> String {
        match self {
            Self::Yes => "Continuous stopping drop off.",
            Self::No => "No continuous stopping drop off.",
            Self::MustPhoneAgency => "Must phone agency to arrange continuous stopping drop off. ",
            Self::MustCoordinateWithDriver => {
                "Must coordinate with driver to arrange continuous stopping drop off. "
            }
        }
        .to_owned()
    }
}

pub type RouteId = Id<Route>;

/// Transit routes. A route is a group of trips that are displayed to riders as a
/// single service.
/// Primary Key: `route_id`
/// See <https://gtfs.org/schedule/reference/#routestxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Unique Primary Key.
    /// Identifies a route.
    #[serde(rename = "route_id")]
    pub id: Id<Route>,

    /// Foreign ID referencing `agency.agency_id`.
    /// Agency for the specified route.
    ///
    /// Conditionally Required:
    /// - **Required** if multiple agencies are defined in agency.txt.
    /// - Recommended otherwise.
    pub agency_id: Option<Id<Agency>>,

    /// Short name of a route. Often a short, abstract identifier
    /// (e.g., "32", "100X", "Green") that riders use to identify a route.
    /// Both `route_short_name` and `route_long_name` may be defined.
    ///
    /// Conditionally Required:
    /// - **Required** if `routes.route_long_name` is empty.
    /// - Recommended if there is a brief service designation. This should be the
    ///   commonly-known passenger name of the service, and should be no longer than
    ///   12 characters.
    #[serde(rename = "route_short_name")]
    pub short_name: Option<String>,

    /// Full name of a route. This name is generally more descriptive than the
    /// `route_short_name` and often includes the route's destination or stop.
    /// Both `route_short_name` and `route_long_name` may be defined.
    ///
    /// Conditionally Required:
    /// - **Required** if `routes.route_short_name` is empty.
    /// - Optional otherwise.
    #[serde(rename = "route_long_name")]
    pub long_name: Option<String>,

    /// Description of a route that provides useful, quality information. Should not
    /// be a duplicate of `route_short_name` or `route_long_name`.
    ///
    /// # Examples
    ///
    /// "A" trains operate between Inwood-207 St, Manhattan and Far Rockaway-Mott
    /// Avenue, Queens at all times. Also from about 6AM until about midnight,
    /// additional "A" trains operate between Inwood-207 St and Lefferts Boulevard
    /// (trains typically alternate between Lefferts Blvd and Far Rockaway).
    #[serde(rename = "route_desc")]
    pub description: Option<String>,

    /// Indicates the type of transportation used on a route.
    #[serde(rename = "route_type")]
    pub kind: RouteType,

    /// URL of a web page about the particular route. Should be different from the
    /// `agency.agency_url` value.
    #[serde(rename = "route_url")]
    pub url: Option<Url>,

    /// Route color designation that matches public facing material.
    /// Defaults to white (`FFFFFF`) when omitted or left empty.
    /// The color difference between `route_color` and `route_text_color` should
    /// provide sufficient contrast when viewed on a black and white screen.
    #[serde(rename = "route_color")]
    pub color: Option<String>,
    //#[serde(rename = "route_color", default = "Color::white")]
    //pub color: Color,
    /// Legible color to use for text drawn against a background of `route_color`.
    /// Defaults to black (`000000`) when omitted or left empty. The color difference
    /// between `route_color` and `route_text_color` should provide sufficient
    /// contrast when viewed on a black and white screen.
    #[serde(rename = "route_text_color")]
    pub text_color: Option<String>,
    //#[serde(rename = "route_text_color", default = "Color::black")]
    //pub text_color: Color,
    /// Orders the routes in a way which is ideal for presentation to customers.
    /// Routes with smaller `route_sort_order` values should be displayed first.
    ///
    /// Type is specified as "Non-negative integer" in the GTFS Schedule reference.
    #[serde(rename = "route_sort_order")]
    pub sort_order: Option<u32>,

    /// Indicates that the rider can board the transit vehicle at any point along the
    /// vehicle’s travel path as described by shapes.txt, on every trip of the route.
    ///
    /// Values for `routes.continuous_pickup` may be overridden by defining values in
    /// `stop_times.continuous_pickup` for specific stop_times along the route.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `stop_times.start_pickup_drop_off_window` or
    ///   `stop_times.end_pickup_drop_off_window` are defined for any trip of this
    ///   route.
    /// - Optional otherwise.
    ///
    /// Defaults to: `ContinuousPickup::No`.
    #[serde(default)]
    pub continuous_pickup: ContinuousPickup,

    /// Indicates that the rider can alight from the transit vehicle at any point
    /// along the vehicle’s travel path as described by shapes.txt, on every trip of
    /// the route.
    ///
    /// Values for `routes.continuous_drop_off` may be overridden by defining values
    /// in `stop_times.continuous_drop_off` for specific `stop_times` along the route.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `stop_times.start_pickup_drop_off_window` or
    ///   `stop_times.end_pickup_drop_off_window` are defined for any trip of this
    ///   route.
    /// - Optional otherwise.
    ///
    /// Defaults to: `ContinuousDropOff::No`.
    #[serde(default)]
    pub continuous_drop_off: ContinuousDropOff,

    /// Identifies a group of routes.
    /// Multiple rows in routes.txt may have the same `network_id`.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if the route_networks.txt file exists.
    /// - Optional otherwise.
    pub network_id: Option<IdString>, // TODO: use proper typed id?
}

impl HasId for Route {
    type IdType = String;
}

impl WithPrimaryKey<RouteId> for Route {
    fn primary_key(&self) -> RouteId {
        self.id.clone()
    }
}
