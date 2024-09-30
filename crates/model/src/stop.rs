use std::cmp;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utility::{
    edit_distance::edit_distance,
    geo::{self, haversine_distance},
    id::{HasId, Id},
    math::sigmoid,
};

use crate::{ExampleData, Mergable, Subject, WithDistance};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StopNameSuggestion {
    #[serde(skip)]
    pub id: Id<Stop>,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(skip)]
    pub parent_id: Option<Id<Stop>>,
    pub location: Option<Location>,
    pub platform_code: Option<String>,
}

impl Stop {
    pub fn latitude(&self) -> Option<f64> {
        self.location.as_ref().map(|location| location.latitude)
    }

    pub fn longitude(&self) -> Option<f64> {
        self.location.as_ref().map(|location| location.longitude)
    }

    pub fn address(&self) -> Option<String> {
        self.location
            .as_ref()
            .and_then(|location| location.address.clone())
    }

    pub fn with_distance_to(
        self,
        latitude: f64,
        longitude: f64,
    ) -> Option<WithDistance<Stop>> {
        let stop_latitude = self.location.as_ref()?.latitude;
        let stop_longitude = self.location.as_ref()?.longitude;
        let distance = geo::haversine_distance(
            latitude,
            longitude,
            stop_latitude,
            stop_longitude,
        );
        Some(WithDistance::new(distance, self))
    }
}

impl HasId for Stop {
    type IdType = String;
}

impl Mergable for Stop {
    fn merge(self, other: Self) -> Self {
        Stop {
            name: other.name.or(self.name),
            description: other.description.or(self.description),
            parent_id: other.parent_id.or(self.parent_id),
            location: self.location.merge(other.location),
            platform_code: other.platform_code.or(self.platform_code),
        }
    }
}

pub const DISTANCE_THRESHOLD_KM: f64 = 0.25;
impl Subject for Stop {
    fn same_subject_as(&self, other: &Self) -> Option<f64> {
        const REPLACE: &[(&str, &[&str])] = &[
            ("hbf", &["hauptbahnhof", "central station"]),
            ("bf", &["bahnhof", "bhf"]),
            ("str", &["straße", "street"]),
        ];
        const GEO_WEIGHT: f64 = 0.5;
        const NAME_WEIGHT: f64 = 0.3;
        const PLATFORM_WIGHT: f64 = 0.1;
        const PARENT_WEIGHT: f64 = 0.1;

        // calculate distance between both stops
        let geo_distance =
            self.location
                .as_ref()
                .zip(other.location.as_ref())
                .map(|(a, b)| {
                    haversine_distance(
                        a.latitude,
                        a.longitude,
                        b.latitude,
                        b.longitude,
                    )
                });
        // avoid further calculation if distance is already too high
        if matches!(geo_distance, Some(x) if x > DISTANCE_THRESHOLD_KM) {
            return None;
        }
        // normalize distance
        let geo_similarity = geo_distance.map(|distance| {
            if distance > DISTANCE_THRESHOLD_KM {
                -1.0
            } else {
                1.0 - (distance / DISTANCE_THRESHOLD_KM)
            }
        });

        // platform codes must be equal if set for both
        let platform_similarity = match (&self.platform_code, &other.platform_code) {
            // both set, but not equal => stops known to be distinct, so return
            // to avoid further calculation.
            (Some(a), Some(b)) if a != b => return None,
            // bot set and equal => positive effect on similarity
            (Some(a), Some(b)) if a == b => 1.0,
            // not set for both => no effect on similarity
            _ => 0.0,
        };

        // parent station must be eqal if set for both
        let parent_similarity = match (&self.parent_id, &other.parent_id) {
            // both set, but not equal => stops known to be distinct, so return
            // to avoid further calculation.
            (Some(a), Some(b)) if a != b => return None,
            // bot set and equal => positive effect on similarity
            (Some(a), Some(b)) if a == b => 1.0,
            // not set for both => no effect on similarity
            _ => 0.0,
        };

        // calculate name similarity
        let names = self.name.as_ref().zip(other.name.as_ref()).map(|(a, b)| {
            [a, b].map(|name| {
                let mut name: String = name
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                for (abbrev, patterns) in REPLACE {
                    for pattern in *patterns {
                        name = name.replace(pattern, abbrev);
                    }
                }
                name
            })
        });
        let name_similarity = names.map(|names| {
            // calculate distance
            let distance = edit_distance(&names[0], &names[1]);
            // normalize
            distance as f64 / cmp::max(names[0].len(), names[1].len()) as f64
        });

        // avoid further caclulation if not enough data to match is available
        if geo_similarity.is_none() && name_similarity.is_none() {
            return None;
        }

        // evaluate overall similarty
        let combined = GEO_WEIGHT * geo_similarity.unwrap_or(0.0)
            + NAME_WEIGHT * name_similarity.unwrap_or(0.0)
            + PLATFORM_WIGHT * platform_similarity
            + PARENT_WEIGHT * parent_similarity;

        Some(sigmoid(combined))
    }
}

impl ExampleData for Stop {
    fn example_data() -> Self {
        Stop {
            name: Some("Bad Malente-Gremsmühlen".to_owned()),
            description: None,
            parent_id: None,
            location: None,
            platform_code: Some("1".to_owned()),
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
}

impl Mergable for Location {
    fn merge(self, other: Self) -> Self {
        Location {
            latitude: other.latitude,
            longitude: other.longitude,
            address: other.address.or(self.address),
        }
    }
}
