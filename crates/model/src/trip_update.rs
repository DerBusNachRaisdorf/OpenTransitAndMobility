use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use utility::id::{HasId, Id};

use crate::{trip::Trip, Mergable};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TripStatus {
    Scheduled,
    Unscheduled,
    Cancelled,
    Added,
    Deleted,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripUpdate {
    pub status: TripStatus,
    pub stops: Vec<StopTimeUpdate>,
    pub timestamp: Option<DateTime<Local>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TripUpdateId {
    pub trip_id: Id<Trip>,
    pub trip_start_date: NaiveDate,
}

impl TripUpdateId {
    pub fn new(trip_id: Id<Trip>, trip_start_date: NaiveDate) -> Self {
        Self {
            trip_id,
            trip_start_date,
        }
    }
}

impl Mergable for TripUpdate {
    fn merge(self, other: Self) -> Self {
        other // TODO: merge appropriate!!
    }
}

impl HasId for TripUpdate {
    type IdType = TripUpdateId;
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopTimeStatus {
    Scheduled,
    Cancelled,
    Added,
    Unknown,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTimeUpdate {
    //pub stop_sequence: i32,
    pub scheduled_stop_sequence: Option<i32>,
    pub arrival_time: Option<DateTime<Local>>,
    pub departure_time: Option<DateTime<Local>>,
    pub status: StopTimeStatus,
}
