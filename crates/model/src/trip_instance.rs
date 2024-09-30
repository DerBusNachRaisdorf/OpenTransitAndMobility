use chrono::{DateTime, Local};
use schemars::JsonSchema;
use serde::Serialize;
use utility::id::Id;

use crate::{
    agency::Agency,
    calendar::Service,
    line::Line,
    stop::{Location, Stop},
    trip::Trip,
    WithId,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TripInstance {
    #[serde(flatten)]
    pub info: TripInstanceInfo,
    pub stops: Vec<StopTimeInstance>,
    pub stop_of_interest: Option<StopTimeInstance>,
    pub line: Option<WithId<Line>>,
    pub agency: Option<WithId<Agency>>,
}

impl TripInstance {
    pub fn get_stop_time_by_sequence(
        &self,
        stop_sequence: i32,
    ) -> Option<StopTimeInstance> {
        for stop in self.stops.iter() {
            if stop.stop_sequence == stop_sequence {
                return Some(stop.clone());
            }
        }
        None
    }

    pub fn sorted(mut trips: Vec<TripInstance>) -> Vec<TripInstance> {
        Self::sort(&mut trips);
        trips
    }

    pub fn sort(trips: &mut Vec<TripInstance>) {
        trips.sort_by(|lhs, rhs| {
            let first = lhs
                .stop_of_interest
                .as_ref()
                .and_then(|soi| soi.departure_time.or(soi.arrival_time));
            let second = rhs
                .stop_of_interest
                .as_ref()
                .and_then(|soi| soi.departure_time.or(soi.arrival_time));
            match (first, second) {
                (Some(first), Some(second)) => first.cmp(&second),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
    }
}

// TODO: skip ids when serializing
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TripInstanceInfo {
    //#[serde(skip)]
    pub trip_id: Id<Trip>,

    #[serde(skip)]
    pub line_id: Id<Line>,

    #[serde(skip)]
    pub service_id: Option<Id<Service>>, // TODO: this should not be optional!

    pub headsign: Option<String>,

    pub short_name: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StopTimeInstance {
    //#[serde(skip)]
    pub stop_sequence: i32,

    #[serde(skip)]
    pub stop_id: Option<Id<Stop>>,

    pub stop_name: Option<String>,

    pub arrival_time: Option<DateTime<Local>>,

    pub departure_time: Option<DateTime<Local>>,

    pub stop_headsign: Option<String>,

    pub interest_flag: bool,

    pub location: Option<Location>,
}
