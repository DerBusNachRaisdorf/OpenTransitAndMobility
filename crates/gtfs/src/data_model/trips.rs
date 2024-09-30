use serde::{Deserialize, Serialize};

use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::id::{HasId, Id};

use crate::database::WithPrimaryKey;

use super::{routes::Route, IdString};

/// Indicates wheelchair accessibility.
/// See <https://gtfs.org/schedule/reference/#tripstxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum WheelchairAccessibility {
    /// No accessibility information for the trip.
    #[default]
    NoAccessibilityInformation = 0,

    /// Vehicle being used on this particular trip can accommodate at least one rider
    /// in a wheelchair.
    CanAccommodateAtLeastOneRiderInWheelchair = 1,

    /// No riders in wheelchairs can be accommodated on this trip.
    CanAccommodateNoRisersInWheelcahirs = 2,
}

impl WheelchairAccessibility {
    pub fn display_text(self) -> String {
        match self {
            Self::NoAccessibilityInformation => "No accessibility information for the trip.",
            Self::CanAccommodateAtLeastOneRiderInWheelchair => {
                "Vehicle being used on this particular trip can accommodate at least \
                 one rider in a wheelchair."
            }
            Self::CanAccommodateNoRisersInWheelcahirs => {
                "No riders in wheelchairs can be accommodated on this trip."
            }
        }
        .to_owned()
    }
}

/// Indicates whether bikes are allowed.
/// See <https://gtfs.org/schedule/reference/#tripstxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum BikesAllowed {
    /// No bike information for the trip.
    #[default]
    NoBikeInformation = 0,

    /// Vehicle being used on this particular trip can accommodate at least one
    /// bicycle.
    CanAccomodateAtLeastOneBicycle = 1,

    /// No bicycles are allowed on this trip.
    NoBicyclesAllowed = 2,
}

/// Indicates the direction of travel for a trip. This field should not be used in
/// routing; it provides a way to separate trips by direction when publishing time
/// tables.
///
/// # Examples
///
/// The `trip_headsign` and `direction_id` fields may be used together to assign a
/// name to travel in each direction for a set of trips. A trips.txt file could
/// contain these records for use in time tables:
/// - `trip_id,...,trip_headsign,direction_id`
/// - `1234,...,Airport,0`
/// - `1505,...,Downtown,1`
///
/// See <https://gtfs.org/schedule/reference/#tripstxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum TravelDirection {
    /// Travel in one direction (e.g. outbound travel).
    TravelInOneDirection = 0,

    /// Travel in the opposite direction (e.g. inbound travel).
    TravelInOppositeDirection = 1,
}

pub type TripId = Id<Trip>;

/// Trips for each route. A trip is a sequence of two or more stops that occur during
/// a specific time period.
/// Primary Key: `trip_id`
/// See <https://gtfs.org/schedule/reference/#tripstxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    /// Unique Primary Key.
    /// Identifies a trip.
    #[serde(rename = "trip_id")]
    pub id: Id<Trip>,

    /// Foreign ID referencing `routes.route_id`.
    /// Identifies a route.
    pub route_id: Id<Route>,

    /// Foreign ID referencing `calendar.service_id` or `calendar_dates.service_id`.
    /// Identifies a set of dates when service is available for one or more routes.
    pub service_id: IdString, // TODO: use proper typed id.

    /// Text that appears on signage identifying the trip's destination to riders.
    /// Should be used to distinguish between different patterns of service on the
    /// same route.
    ///
    /// If the headsign changes during a trip, values for `trip_headsign` may be
    /// overridden by defining values in `stop_times.stop_headsign` for specific
    /// `stop_times` along the trip.
    #[serde(rename = "trip_headsign")]
    pub headsign: Option<String>,

    /// Public facing text used to identify the trip to riders, for instance, to
    /// identify train numbers for commuter rail trips. If riders do not commonly rely
    /// on trip names, `trip_short_name` should be empty. A `trip_short_name` value,
    /// if provided, should uniquely identify a trip within a service day; it should
    /// not be used for destination names or limited/express designations.
    #[serde(rename = "trip_short_name")]
    pub short_name: Option<String>,

    /// Indicates the direction of travel for a trip. This field should not be used in
    /// routing; it provides a way to separate trips by direction when publishing time
    /// tables.
    ///
    /// # Examples
    ///
    /// The `trip_headsign` and `direction_id` fields may be used together to assign a
    /// name to travel in each direction for a set of trips. A trips.txt file could
    /// contain these records for use in time tables:
    /// - `trip_id,...,trip_headsign,direction_id`
    /// - `1234,...,Airport,0`
    /// - `1505,...,Downtown,1`
    #[serde(rename = "direction_id")]
    pub direction: Option<TravelDirection>,

    /// Identifies the block to which the trip belongs.
    /// A block consists of a single trip or many sequential trips made using the same
    /// vehicle, defined by shared service days and `block_id`. A `block_id` may have
    /// trips with different service days, making distinct blocks.
    /// To provide in-seat transfers information, transfers of `transfer_type 4`
    /// should be provided instead.
    pub block_id: Option<IdString>,

    /// Foreign ID referencing `shapes.shape_id`.
    /// Identifies a geospatial shape describing the vehicle travel path for a trip.
    ///
    /// Conditionally Required:
    /// - **Required** if the trip has a continuous pickup or drop-off behavior
    ///   defined either in routes.txt or in stop_times.txt.
    /// - Optional otherwise.
    pub shape_id: Option<IdString>, // TODO: user proper typed id.

    /// Indicates wheelchair accessibility.
    /// Defaults to: `WheelchairAccessibility::NoAccessibilityInformation`
    #[serde(default)]
    pub wheelchair_accessible: WheelchairAccessibility,

    /// Indicates whether bikes are allowed.
    /// Defaults to: `BikesAllowed::NoBikeInformation`
    #[serde(default)]
    pub bikes_allowed: BikesAllowed,
}

impl HasId for Trip {
    type IdType = String;
}

impl WithPrimaryKey<TripId> for Trip {
    fn primary_key(&self) -> TripId {
        self.id.clone()
    }
}
