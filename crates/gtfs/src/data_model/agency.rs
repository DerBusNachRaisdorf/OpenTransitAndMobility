use serde::{Deserialize, Serialize};
use utility::id::{HasId, Id};

use crate::database::WithPrimaryKey;

use super::{Email, LanguageCode, PhoneNumber, Timezone, Url};

pub type AgencyId = Id<Agency>;

/// Transit agencies with service represented in this dataset.
/// Primary Key: `agency_id`
/// See <https://gtfs.org/schedule/reference/#agencytxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agency {
    /// Unique Primary Key.
    /// Identifies a transit brand which is often synonymous with a transit agency.
    /// Note that in some cases, such as when a single agency operates multiple
    /// separate services, agencies and brands are distinct. This document uses the
    /// term "agency" in place of "brand". A dataset may contain data from multiple
    /// agencies.
    ///
    /// Conditionally Required:
    /// - **Required** when the dataset contains data for multiple transit agencies.
    /// - Recommended otherwise.
    #[serde(rename = "agency_id")]
    pub id: Option<Id<Agency>>,

    /// Full name of the transit agency.
    #[serde(rename = "agency_name")]
    pub name: String,

    /// URL of the transit agency.
    #[serde(rename = "agency_url")]
    pub url: Url,

    /// Timezone where the transit agency is located. If multiple agencies are
    /// specified in the dataset, each must have the same `agency_timezone`.
    #[serde(rename = "agency_timezone")]
    pub timezone: Timezone, // TODO Type: Timezone

    /// Primary language used by this transit agency. Should be provided to help GTFS
    /// consumers choose capitalization rules and other language-specific settings for
    /// the dataset.
    #[serde(rename = "agency_lang")]
    pub language_code: Option<LanguageCode>,

    /// A voice telephone number for the specified agency. This field is a string
    /// value that presents the telephone number as typical for the agency's service
    /// area. It may contain punctuation marks to group the digits of the number.
    /// Dialable text (for example, TriMet's "503-238-RIDE") is permitted, but the
    /// field must not contain any other descriptive text.
    #[serde(rename = "agency_phone")]
    pub phone_number: Option<PhoneNumber>,

    /// URL of a web page that allows a rider to purchase tickets or other fare
    /// instruments for that agency online.
    #[serde(rename = "agency_fare_url")]
    pub fare_url: Option<Url>,

    /// Email address actively monitored by the agency’s customer service department.
    /// This email address should be a direct contact point where transit riders can
    /// reach a customer service representative at the agency.
    #[serde(rename = "agency_email")]
    pub email: Option<Email>,
}

impl HasId for Agency {
    type IdType = String;
}

impl WithPrimaryKey<Option<AgencyId>> for Agency {
    fn primary_key(&self) -> Option<AgencyId> {
        self.id.clone()
    }
}
