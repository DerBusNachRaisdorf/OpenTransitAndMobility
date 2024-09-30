use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utility::id::HasId;

use crate::ExampleData;
use crate::Mergable;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Agency {
    pub name: String,
    pub website: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub fare_url: Option<String>,
}

impl HasId for Agency {
    type IdType = String;
}

impl Mergable for Agency {
    fn merge(self, other: Self) -> Self {
        Self {
            name: other.name,
            website: other.website,
            phone_number: other.phone_number.or(self.phone_number),
            email: other.email.or(self.email),
            fare_url: other.fare_url.or(self.fare_url),
        }
    }
}

impl ExampleData for Agency {
    fn example_data() -> Self {
        Self {
            name: "erixx schleswig".to_owned(),
            website: "erixx-schleswig.de".to_owned(),
            phone_number: Some("04522 42069".to_owned()),
            email: Some("some@email.com".to_owned()),
            fare_url: Some("buy.some-tickets.com".to_owned()),
        }
    }
}
