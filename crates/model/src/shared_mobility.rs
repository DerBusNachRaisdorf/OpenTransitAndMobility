use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utility::geo;
use utility::id::HasId;

use crate::Mergable;
use crate::WithDistance;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedMobilityStation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub capacity: u32,
    pub rental_uris: RentalUris,
    pub status: Option<Status>,
}

impl HasId for SharedMobilityStation {
    type IdType = String;
}

impl Mergable for SharedMobilityStation {
    fn merge(self, other: Self) -> Self {
        Self {
            name: other.name,
            latitude: other.latitude,
            longitude: other.longitude,
            capacity: other.capacity,
            rental_uris: RentalUris {
                android: other.rental_uris.android.or(self.rental_uris.android),
                ios: other.rental_uris.ios.or(self.rental_uris.ios),
                web: other.rental_uris.web.or(self.rental_uris.web),
            },
            status: other.status,
        }
    }
}

impl SharedMobilityStation {
    pub fn with_distance_to(
        self,
        latitude: f64,
        longitude: f64,
    ) -> Option<WithDistance<Self>> {
        let distance = geo::haversine_distance(
            latitude,
            longitude,
            self.latitude,
            self.longitude,
        );
        Some(WithDistance::new(distance, self))
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RentalUris {
    pub android: Option<String>,
    pub ios: Option<String>,
    pub web: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub num_bikes_available: u32,
    pub num_docks_available: u32,
    // TODO: hier detailierte informationen zu Fahrzeugtypen etc.
}
