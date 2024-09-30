use crate::serde::default_if_empty;
use chrono::Duration;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::id::Id;
use utility::serde::duration;

use crate::database::WithPrimaryKey;

use super::{
    routes::{ContinuousDropOff, ContinuousPickup},
    stops::Stop,
    trips::{Trip, TripId},
    IdString, Time,
};

/// Indicates pickup method.
/// See <https://gtfs.org/schedule/reference/#stop_timestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum PickupMethod {
    /// Regularly scheduled pickup.
    #[default]
    RegularlyScheduled = 0,

    /// No pickup available.
    NotAvailable = 1,

    /// Must phone agency to arrange pickup.
    MustPhoneAgency = 2,

    /// Must coordinate with driver to arrange pickup.
    MustCoordinateWithDriver = 3,
}

impl PickupMethod {
    pub fn display_text(self) -> String {
        match self {
            Self::RegularlyScheduled => "Regularly scheduled pickup.",
            Self::NotAvailable => "No pickup available.",
            Self::MustPhoneAgency => "Must phone agency to arrange pickup.",
            Self::MustCoordinateWithDriver => {
                "Must coordinate with driver to arrange pickup."
            }
        }
        .to_owned()
    }
}

/// Indicates drop off method.
/// See <https://gtfs.org/schedule/reference/#stop_timestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum DropOffMethod {
    /// Regularly scheduled drop off.
    #[default]
    RegularlyScheduled = 0,

    /// No drop off available.
    NotAvailable = 1,

    /// Must phone agency to arrange drop off.
    MustPhoneAgency = 2,

    /// Must coordinate with driver to arrange drop off.
    MustCoordinateWithDriver = 3,
}

impl DropOffMethod {
    pub fn display_text(self) -> String {
        match self {
            Self::RegularlyScheduled => "Regularly scheduled drop off.",
            Self::NotAvailable => "No drop off available.",
            Self::MustPhoneAgency => "Must phone agency to arrange drop off.",
            Self::MustCoordinateWithDriver => {
                "Must coordinate with driver to arrange drop off."
            }
        }
        .to_owned()
    }
}

pub type StopTimeKey = (TripId, u32);

/// Times that a vehicle arrives at and departs from stops for each trip.
/// Primary Key: `(trip_id, stop_sequence)`
/// See <https://gtfs.org/schedule/reference/#stop_timestxt>
#[derive(Debug, Clone, Deserialize)]
pub struct StopTime {
    /// Foreign ID referencing `trips.trip_id`.
    /// Identifies a trip.
    pub trip_id: Id<Trip>,

    /// Arrival time at the stop (defined by `stop_times.stop_id`) for a specific trip
    /// (defined by `stop_times.trip_id`) in the time zone specified by
    /// `agency.agency_timezone`, not `stops.stop_timezone`.
    ///
    /// If there are not separate times for arrival and departure at a stop,
    /// `arrival_time` and `departure_time` should be the same.
    ///
    /// For times occurring after midnight on the service day, enter the time as a
    /// value greater than `24:00:00` in `HH:MM:SS`.
    ///
    /// If exact arrival and departure times (`timepoint=1` or empty) are not
    /// available, estimated or interpolated arrival and departure times
    /// (`timepoint=0`) should be provided.
    ///
    /// Conditionally Required:
    /// - **Required** for the first and last stop in a trip
    ///   (defined by `stop_times.stop_sequence`).
    /// - **Required** for `timepoint=1`.
    /// - **Forbidden** when `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    #[serde(deserialize_with = "duration::deserialize_option")]
    pub arrival_time: Option<Duration>,

    /// Departure time from the stop (defined by `stop_times.stop_id`) for a specific
    /// trip (defined by `stop_times.trip_id`) in the time zone specified by
    /// `agency.agency_timezone`, not `stops.stop_timezone`.
    ///
    /// If there are not separate times for arrival and departure at a stop,
    /// `arrival_time` and `departure_time` should be the same.
    ///
    /// For times occurring after midnight on the service day, enter the time as a
    /// value greater than `24:00:00` in `HH:MM:SS`.
    ///
    /// If exact arrival and departure times (`timepoint=1` or empty) are not
    /// available, estimated or interpolated arrival and departure times
    /// (`timepoint=0`) should be provided.
    ///
    /// Conditionally Required:
    /// - **Required** for `timepoint=1`.
    /// - **Forbidden** when `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    #[serde(deserialize_with = "duration::deserialize_option")]
    pub departure_time: Option<Duration>,

    /// Foreign ID referencing `stops.stop_id`.
    /// dentifies the serviced stop. All stops serviced during a trip must have a
    /// record in stop_times.txt. Referenced locations must be stops/platforms, i.e.
    /// their `stops.location_type` value must be `0` or empty. A stop may be serviced
    /// multiple times in the same trip, and multiple trips and routes may service the
    /// same stop.
    ///
    /// On-demand service using stops should be referenced in the sequence in which
    /// service is available at those stops. A data consumer should assume that travel
    /// is possible from one stop or location to any stop or location later in the
    /// trip, provided that the `pickup`/`drop_off_type` of each `stop_time` and the
    /// time constraints of each `start`/`end_pickup_drop_off_window` do not forbid
    /// it.
    ///
    /// Conditionally Required:
    /// - **Required** if `stop_times.location_group_id` AND `stop_times.location_id`
    ///   are NOT defined.
    /// - **Forbidden** if `stop_times.location_group_id` or `stop_times.location_id`
    ///   are defined.
    pub stop_id: Option<Id<Stop>>,

    /// Foreign ID referencing `location_groups.location_group_id`.
    /// Identifies the serviced location group that indicates groups of stops where
    /// riders may request pickup or drop off. All location groups serviced during a
    /// trip must have a record in stop_times.txt. Multiple trips and routes may
    /// service the same location group.
    ///
    /// On-demand service using location groups should be referenced in the sequence
    /// in which service is available at those location groups. A data consumer should
    /// assume that travel is possible from one stop or location to any stop or
    /// location later in the trip, provided that the `pickup`/`drop_off_type` of each
    /// `stop_time` and the time constraints of each
    /// `start`/`end_pickup_drop_off_window` do not forbid it.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `stop_times.stop_id` or `stop_times.location_id` are
    ///   defined.
    pub location_group_id: Option<IdString>, // TODO: use proper typed Id.

    /// Foreign ID referencing `id` from `locations.geojson`.
    /// Identifies the GeoJSON location that corresponds to serviced zone where riders
    /// may request pickup or drop off. All GeoJSON locations serviced during a trip
    /// must have a record in stop_times.txt. Multiple trips and routes may service
    /// the same GeoJSON location.
    ///
    /// On-demand service within locations should be referenced in the sequence in
    /// which service is available in those locations. A data consumer should assume
    /// that travel is possible from one stop or location to any stop or location
    /// later in the trip, provided that the `pickup`/`drop_off_type` of each
    /// `stop_time` and the time constraints of each
    /// `start`/`end_pickup_drop_off_window` do not forbid it.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `stop_times.stop_id` or `stop_times.location_group_id` are
    ///   defined.
    pub location_id: Option<IdString>, // TODO: use proper typed Id??

    /// Order of stops, location groups, or GeoJSON locations for a particular trip.
    /// The values must increase along the trip but do not need to be consecutive.
    ///
    /// # Example
    ///
    /// The first location on the trip could have a `stop_sequence=1`, the second
    /// location on the trip could have a `stop_sequence=23`, the third location could
    /// have a `stop_sequence=40`, and so on.
    ///
    /// Travel within the same location group or GeoJSON location requires two records
    /// in stop_times.txt with the same `location_group_id` or `location_id`.
    ///
    /// Type is specified as "Non-negative integer" in the GTFS Schedule reference.
    pub stop_sequence: u32,

    /// Text that appears on signage identifying the trip's destination to riders.
    /// This field overrides the default `trips.trip_headsign` when the headsign
    /// changes between stops. If the headsign is displayed for an entire trip,
    /// `trips.trip_headsign` should be used instead.
    ///
    /// A `stop_headsign` value specified for one `stop_time` does not apply to
    /// subsequent `stop_times` in the same trip. If you want to override the
    /// `trip_headsign` for multiple `stop_times` in the same trip, the
    /// `stop_headsign` value must be repeated in each `stop_time` row.
    pub stop_headsign: Option<String>,

    /// Time that on-demand service becomes available in a GeoJSON location,
    /// location group, or stop.
    ///
    /// Conditionally Required:
    /// - **Required** if `stop_times.location_group_id` or `stop_times.location_id`
    ///   is defined.
    /// - **Required** if `end_pickup_drop_off_window` is defined.
    /// - **Forbidden** if `arrival_time` or `departure_time` is defined.
    /// - Optional otherwise.
    pub start_pickup_drop_off_window: Option<Time>,

    /// Time that on-demand service ends in a GeoJSON location, location group,
    /// or stop.
    ///
    /// Conditionally Required:
    /// - **Required** if `stop_times.location_group_id` or `stop_times.location_id`
    ///   is defined.
    /// - **Required** if `start_pickup_drop_off_window` is defined.
    /// - **Forbidden** if `arrival_time` or `departure_time` is defined.
    /// - Optional otherwise.
    pub end_pickup_drop_off_window: Option<Time>,

    /// Indicates pickup method.
    /// Conditionally Forbidden:
    /// - `pickup_type=0` **forbidden** if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - `pickup_type=3` **forbidden** if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    ///
    /// Defaults to `PickupMethod::RegularlyScheduled`.
    //#[serde(default)]
    #[serde(deserialize_with = "default_if_empty")]
    pub pickup_type: PickupMethod,

    /// Indicates drop off method.
    ///
    /// Conditionally Forbidden:
    /// - `drop_off_type=0` **forbidden** if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    ///
    /// Defaults to: `DropOffMethod::RegularlyScheduled`.
    //#[serde(default)]
    #[serde(deserialize_with = "default_if_empty")]
    pub drop_off_type: DropOffMethod,

    /// Indicates that the rider can board the transit vehicle at any point along the
    /// vehicle’s travel path as described by shapes.txt, from this `stop_time` to the
    /// next `stop_time` in the trip’s `stop_sequence`.
    ///
    /// If this field is populated, it overrides any continuous pickup behavior
    /// defined in routes.txt. If this field is empty, the `stop_time` inherits any
    /// continuous pickup behavior defined in routes.txt.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub continuous_pickup: Option<ContinuousPickup>,

    /// Indicates that the rider can alight from the transit vehicle at any point
    /// along the vehicle’s travel path as described by shapes.txt, from this
    /// `stop_time` to the next `stop_time` in the trip’s stop_sequence.
    ///
    /// If this field is populated, it overrides any continuous drop-off behavior
    /// defined in routes.txt. If this field is empty, the `stop_time` inherits any
    /// continuous drop-off behavior defined in routes.txt.
    ///
    /// Conditionally Forbidden:
    /// - **Forbidden** if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub continuous_drop_off: Option<ContinuousDropOff>,

    /// Actual distance traveled along the associated shape, from the first stop to
    /// the stop specified in this record. This field specifies how much of the shape
    /// to draw between any two stops during a trip. Must be in the same units used in
    /// shapes.txt. Values used for `shape_dist_traveled` must increase along with
    /// `stop_sequence`; they must not be used to show reverse travel along a route.
    ///
    /// Recommended for routes that have looping or inlining (the vehicle crosses or
    /// travels over the same portion of alignment in one trip).
    /// See `shapes.shape_dist_traveled`.
    ///
    /// # Example
    ///
    /// If a bus travels a distance of 5.25 kilometers from the start of the shape to
    /// the stop, `shape_dist_traveled=5.25`.
    ///
    /// Type specified as Non-negative float in GTFS Schedule specification.
    #[serde(rename = "shape_dist_traveled")]
    pub shape_distance_traveled: Option<f64>,

    /// Identifies the boarding booking rule at this stop time.
    ///
    /// Recommended when `pickup_type=2`.
    pub pickup_booking_rule_id: Option<IdString>,

    /// Identifies the alighting booking rule at this stop time.
    ///
    /// Recommended when `drop_off_type=2`.
    pub drop_off_booking_rule_id: Option<IdString>,
}

impl WithPrimaryKey<StopTimeKey> for StopTime {
    fn primary_key(&self) -> StopTimeKey {
        (self.trip_id.clone(), self.stop_sequence)
    }
}
