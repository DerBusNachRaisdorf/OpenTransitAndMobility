use serde::{Deserialize, Serialize};
use utility::id::Id;

use super::{routes::Route, stops::Stop, trips::TripId};

/// Indicates the type of connection for the specified `(from_stop_id, to_stop_id)`
/// pair.
///
/// See <https://gtfs.org/schedule/reference/#transferstxt>
///
/// # Linked trips
///
/// The following applies to `transfer_type=4` and `=5`, which are used to link trips
/// together, with or without in-seats transfers.
///
/// The trips linked together **MUST** be operated by the same vehicle. The vehicle
/// **MAY** be coupled to, or uncoupled from, other vehicles.
///
/// If both a linked trips transfer and a `block_id` are provided and they produce
/// conflicting results, then the linked trips transfer shall be used.
///
/// The last stop of `from_trip_id` **SHOULD** be geographically close to the first
/// stop of `to_trip_id`, and the last arrival time of `from_trip_id` **SHOULD** be
/// prior but close to the first departure time of `to_trip_id`. The last arrival time
/// of `from_trip_id` **MAY** be later than the first departure time of `to_trip_id`
/// in case the `to_trip_id` trip is occurring the subsequent service day.
///
/// Trips **MAY** be linked 1-to-1 in the regular case, but **MAY** also be linked
/// 1-to-n, n-to-1, or n-to-n to represent more complex trip continuations.
/// For example, two train trips (trip A and trip B in the diagram below) can merge
/// into a single train trip (trip C) after a vehicle coupling operation at a common
/// station:
///
/// - In a 1-to-n continuation, the `trips.service_id` for each `to_trip_id` **MUST**
///   be identical.
/// - In an n-to-1 continuation, the `trips.service_id` for each `from_trip_id`
///   **MUST** be identical.
/// - n-to-n continuations must respect both constraints.
/// - Trips may be linked together as part of multiple distinct continuations,
///   provided that the `trip.service_id` **MUST NOT** overlap on any day of service.
///
/// Trip A
/// ───────────────────\
///                     \    Trip C
///                     ─────────────
/// Trip B              /
/// ───────────────────/
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    /// Recommended transfer point between routes.
    #[default]
    RecommendedTransferPoint = 0,

    /// Timed transfer point between two routes. The departing vehicle is expected to
    /// wait for the arriving one and leave sufficient time for a rider to transfer
    /// between routes.
    TimedTransferPoint = 1,

    /// Transfer requires a minimum amount of time between arrival and departure to
    /// ensure a connection. The time required to transfer is specified by
    /// `min_transfer_time`.
    RequiresMinimumAmountOfTime = 2,

    /// Transfers are not possible between routes at the location.
    NotPossible = 3,

    /// Passengers can transfer from one trip to another by staying onboard the same
    /// vehicle (an "in-seat transfer").
    InSeatTransfer = 4,

    /// In-seat transfers are not allowed between sequential trips.
    /// The passenger must alight from the vehicle and re-board.
    InSeatTrnasferNotAllowed = 5,
}

/// Rules for making connections at transfer points between routes.
///
/// Primary key
/// `(from_stop_id, to_stop_id, from_trip_id, to_trip_id, from_route_id, to_route_id)`
///
/// When calculating an itinerary, GTFS-consuming applications interpolate transfers
/// based on allowable time and stop proximity. Transfers.txt specifies additional
/// rules and overrides for selected transfers.
///
/// Fields `from_trip_id`, `to_trip_id`, `from_route_id` and `to_route_id` allow
/// higher orders of specificity for transfer rules. Along with `from_stop_id` and
/// `to_stop_id`, the ranking of specificity is as follows:
///
/// 1. Both `trip_ids` defined: `from_trip_id` and `to_trip_id`.
/// 2. One `trip_id` and `route_id` set defined: (`from_trip_id` and `to_route_id`) or
///    (`from_route_id` and `to_trip_id`).
/// 3. One `trip_id` defined: `from_trip_id` or `to_trip_id`.
/// 4. Both `route_ids` defined: `from_route_id` and `to_route_id`.
/// 5. One `route_id` defined: `from_route_id` or `to_route_id`.
/// 6. Only `from_stop_id` and `to_stop_id` defined: no route or trip related fields
///    set.
///
/// For a given ordered pair of arriving trip and departing trip, the transfer with
/// the greatest specificity that applies between these two trips is chosen.
/// For any pair of trips, there should not be two transfers with equally maximal
/// specificity that could apply.
///
/// See <https://gtfs.org/schedule/reference/#transferstxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransfersRow {
    /// Foreign ID referencing `stops.stop_id`.
    /// Identifies a stop or station where a connection between routes begins.
    /// If this field refers to a station, the transfer rule applies to all its
    /// child stops.
    /// Refering to a station is **forbiden** for `transfer_types` `4` and `5`.
    ///
    /// **Conditionally Required**: see `TransferRow` doc.
    pub from_stop_id: Option<Id<Stop>>,

    /// Foreign ID referencing `stops.stop_id`.
    /// Identifies a stop or station where a connection between routes ends.
    /// If this field refers to a station, the transfer rule applies to all
    /// child stops.
    /// Refering to a station is **forbiden** for `transfer_types` `4` and `5`.
    ///
    /// **Conditionally Required**: see `TransferRow` doc.
    pub to_stop_id: Option<Id<Stop>>,

    /// Foreign ID referencing `routes.route_id`.
    /// Identifies a route where a connection begins.
    ///
    /// If `from_route_id` is defined, the transfer will apply to the arriving trip on
    /// the route for the given `from_stop_id``.
    ///
    /// If both `from_trip_id` and `from_route_id` are defined, the `trip_id` must
    /// belong to the `route_id`, and `from_trip_id` will take precedence.
    pub from_route_id: Option<Id<Route>>,

    /// Foreign ID referencing `routes.route_id`
    /// Identifies a route where a connection ends.
    ///
    /// If `to_route_id` is defined, the transfer will apply to the departing trip on
    /// the route for the given `to_stop_id`.
    ///
    /// If both `to_trip_id` and `to_route_id` are defined, the `trip_id` must belong
    /// to the `route_id`, and `to_trip_id` will take precedence.
    pub to_route_id: Option<Id<Route>>,

    /// Identifies a trip where a connection between routes begins.
    ///
    /// If `from_trip_id` is defined, the transfer will apply to the arriving trip for
    /// the given `from_stop_id`.
    ///
    /// If both `from_trip_id` and `from_route_id` are defined, the `trip_id` must
    /// belong to the `route_id`, and `from_trip_id` will take precedence.
    /// **REQUIRED** if `transfer_type` is `4` or `5`.
    pub from_trip_id: Option<TripId>,

    /// Identifies a trip where a connection between routes ends.
    ///
    /// If `to_trip_id` is defined, the transfer will apply to the departing trip for
    /// the given `to_stop_id`.
    ///
    /// If both `to_trip_id` and `to_route_id` are defined, the `trip_id` must belong
    /// to the `route_id`, and `to_trip_id` will take precedence.
    /// **REQUIRED** if `transfer_type` is `4` or `5`.
    pub to_trip_id: Option<TripId>,

    /// Indicates the type of connection for the specified
    /// `(from_stop_id, to_stop_id)` pair.
    //
    // Defaults to: `TransferType::RecommendedTransferPoint`.
    #[serde(rename = "transfer_type", default)]
    pub kind: TransferType,

    /// Amount of time, in seconds, that must be available to permit a transfer
    /// between routes at the specified stops. The `min_transfer_time` should be
    /// sufficient to permit a typical rider to move between the two stops, including
    /// buffer time to allow for schedule variance on each route.
    ///
    /// Type specified as Non-negative integer in GTFS Schedule reference.
    #[serde(rename = "min_transfer_time")]
    pub minimum_transfer_time: Option<u32>,
}
