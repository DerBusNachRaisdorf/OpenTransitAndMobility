use chrono::Duration;
use schemars::JsonSchema;
use serde::Serialize;
use utility::id::{HasId, Id};
use utility::serde::duration;

use crate::ExampleData;
use crate::{calendar::Service, line::Line, stop::Stop, Mergable};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Trip {
    #[serde(skip)]
    pub line_id: Id<Line>,
    #[serde(skip)]
    pub service_id: Option<Id<Service>>, // TODO: this sould not be optional!
    pub headsign: Option<String>,
    pub short_name: Option<String>,
    pub stops: Vec<StopTime>,
}

impl HasId for Trip {
    type IdType = String;
}

impl Mergable for Trip {
    fn merge(self, other: Self) -> Self {
        Self {
            line_id: other.line_id,
            service_id: other.service_id,
            headsign: other.headsign.or(self.headsign),
            short_name: other.short_name.or(self.short_name),
            stops: other.stops, // TODO: merge strategy
        }
    }
}

impl ExampleData for Trip {
    fn example_data() -> Self {
        Self {
            line_id: Id::new("erixx-re83".to_owned()),
            service_id: Some(Id::new(123)),
            headsign: Some("Kiel Hbf".to_owned()),
            short_name: Some("Lübeck-Kiel".to_owned()),
            stops: vec![
                // TODO!
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StopTime {
    pub stop_sequence: i32,

    pub stop_id: Option<Id<Stop>>,

    /// arrival time as a duration since midnight. this is because
    /// times greates than 24:00:00 are allowed to represent a time at the next day.
    #[serde(serialize_with = "duration::serialize_option")]
    #[schemars(schema_with = "duration::schema")]
    pub arrival_time: Option<Duration>,

    /// departure time as a duration since midnight. this is because
    /// times greates than 24:00:00 are allowed to represent a time at the next day.
    #[serde(serialize_with = "duration::serialize_option")]
    #[schemars(schema_with = "duration::schema_option")]
    pub departure_time: Option<Duration>,

    pub stop_headsign: Option<String>,
}

impl Mergable for StopTime {
    fn merge(self, other: Self) -> Self {
        Self {
            stop_sequence: other.stop_sequence,
            stop_id: other.stop_id.or(self.stop_id),
            arrival_time: other.arrival_time.or(self.arrival_time),
            departure_time: other.departure_time.or(self.departure_time),
            stop_headsign: other.stop_headsign.or(self.stop_headsign),
        }
    }
}
