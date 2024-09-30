use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::{
    id::{HasId, Id},
    serde::date_time::deserialize_yyyymmdd,
};

/// Indicates whether the service operates. Note that exceptions for particular dates
/// may be listed in calendar_dates.txt.
/// See <https://gtfs.org/schedule/reference/#calendartxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum ServiceAvailability {
    Unavailable = 0,
    Available = 1,
}

impl ServiceAvailability {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

impl Into<model::calendar::ServiceAvailability> for ServiceAvailability {
    fn into(self) -> model::calendar::ServiceAvailability {
        match self {
            Self::Available => model::calendar::ServiceAvailability::Available,
            Self::Unavailable => model::calendar::ServiceAvailability::Unavailable,
        }
    }
}

/// Service dates specified using a weekly schedule with start and end dates.
///
/// Conditionally Required:
/// - **Required** unless all dates of service are defined in calendar_dates.txt.
/// - Optional otherwise.
///
/// See <https://gtfs.org/schedule/reference/#calendartxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarRow {
    /// Unique Primary Key.
    /// Identifies a set of dates when service is available for one or more routes.
    pub service_id: Id<CalendarRow>, // TODO: proper ID type??

    /// Indicates whether the service operates on all Mondays in the date range
    /// specified by the start_date and end_date fields. Note that exceptions for
    /// particular dates may be listed in calendar_dates.txt. Valid options are:
    ///
    /// `ServiceAvailability::Available` - Service is available for all Mondays in the
    ///                                    date range.
    /// `ServiceAvailability::Unavailable` - Service is not available for Mondays in
    ///                                      the date range.
    pub monday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Tuesdays.
    pub tuesday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Wednesday.
    pub wednesday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Thursday
    pub thursday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Friday.
    pub friday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Saturday.
    pub saturday: ServiceAvailability,

    /// Functions in the same way as `monday` except applies to Sunday.
    pub sunday: ServiceAvailability,

    /// Start service day for the service interval.
    #[serde(deserialize_with = "deserialize_yyyymmdd")]
    pub start_date: chrono::NaiveDate,

    /// End service day for the service interval. This service day is included in the
    /// interval.
    #[serde(deserialize_with = "deserialize_yyyymmdd")]
    pub end_date: chrono::NaiveDate,
}

impl HasId for CalendarRow {
    type IdType = String;
}
