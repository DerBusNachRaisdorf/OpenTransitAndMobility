use serde::{Deserialize, Serialize};

use super::{trips::TripId, Time};

/// Indicates the type of service for a trip. See the file description for more
/// information.
/// See <https://gtfs.org/schedule/reference/#frequenciestxt>
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum TypeOfTripService {
    /// Frequency-based trips.
    #[default]
    FrequencyBased = 0,

    /// Schedule-based trips with the exact same headway throughout the day.
    /// In this case the `end_time` value must be greater than the last desired trip
    /// `start_time` but less than the last desired trip `start_time` + `headway_secs`.
    ScheduleBased = 1,
}

/// Headway (time between trips) for headway-based service or a compressed
/// representation of fixed-schedule service.
///
/// Primary key `(trip_id, start_time)`
///
/// Frequencies.txt represents trips that operate on regular headways
/// (time between trips).
/// This file may be used to represent two different types of service.
///
/// - Frequency-based service (`exact_times=0`) in which service does not follow a
///   schedule throughout the day. Instead, operators attempt to strictly maintain
///   predetermined headways for trips.
/// - A compressed representation of schedule-based service (`exact_times=1?) that has
///   the exact same headway for trips over specified time period(s).
///   In schedule-based service operators try to strictly adhere to a schedule.
///
/// See <https://gtfs.org/schedule/reference/#frequenciestxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency {
    /// Foreign ID referncing `trips.trip_id`.
    /// Identifies a trip to which the specified headway of service applies.
    pub trip_id: TripId,

    /// Time at which the first vehicle departs from the first stop of the trip with
    /// the specified headway.
    pub start_time: Time,

    /// Time at which service changes to a different headway (or ceases) at the first
    /// stop in the trip.
    pub end_time: Time,

    /// Time, in seconds, between departures from the same stop (headway) for the
    /// trip, during the time interval specified by `start_time` and `end_time`.
    /// Multiple headways may be defined for the same trip, but must not overlap.
    /// New headways may start at the exact time the previous headway ends.
    ///
    /// Type specified as Positive integer in GTFS Schedule reference.
    #[serde(rename = "headway_secs")]
    pub headway_seconds: u32,

    /// Indicates the type of service for a trip. See the file description for more
    /// information.
    ///
    /// Defaults to: `TypeOfTripService::FrequencyBased`.
    #[serde(default)]
    pub exact_times: TypeOfTripService,
}
