use model::calendar::ServiceExceptionType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utility::id::Id;
use utility::serde::date_time::deserialize_yyyymmdd;

use super::calendar::CalendarRow;

/// Indicates whether service is available on the date specified in the date field.
/// See <https://gtfs.org/schedule/reference/#calendar_datestxt>
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum ExceptionType {
    /// Service has been added for the specified date.
    ServiceAdded = 1,

    /// Service has been removed for the specified date.
    ServiceRemoved = 2,
}

impl Into<ServiceExceptionType> for ExceptionType {
    fn into(self) -> ServiceExceptionType {
        match self {
            Self::ServiceAdded => ServiceExceptionType::Added,
            Self::ServiceRemoved => ServiceExceptionType::Removed,
        }
    }
}

/// Exceptions for the services defined in the calendar.txt.
///
/// Conditionally Required:
/// - **Required** if calendar.txt is omitted. In which case calendar_dates.txt must contain all dates of service.
/// - Optional otherwise.
///
/// Primary key (service_id, date)
///
/// The calendar_dates.txt table explicitly activates or disables service by date.
/// It may be used in two ways.
///
/// - Recommended: Use calendar_dates.txt in conjunction with calendar.txt to define
///   exceptions to the default service patterns defined in calendar.txt. If service
///   is generally regular, with a few changes on explicit dates (for instance, to
///   accommodate special event services, or a school schedule), this is a good
///   approach. In this case `calendar_dates.service_id` is a foreign ID referencing
///   `calendar.service_id`.
/// - Alternate: Omit calendar.txt, and specify each date of service in
///   calendar_dates.txt. This allows for considerable service variation and
///   accommodates service without normal weekly schedules.
///   In this case `service_id is` an ID.
///
/// See <https://gtfs.org/schedule/reference/#calendar_datestxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDate {
    /// Foreign ID referencing `calendar.service_id` or ID.
    /// Identifies a set of dates when a service exception occurs for one or more
    /// routes. Each `(service_id, date)` pair may only appear once in
    /// calendar_dates.txt if using calendar.txt and calendar_dates.txt in
    /// conjunction. If a `service_id` value appears in both calendar.txt and
    /// calendar_dates.txt, the information in calendar_dates.txt modifies the service
    /// information specified in calendar.txt.
    pub service_id: Id<CalendarRow>,

    /// Date when service exception occurs.
    #[serde(deserialize_with = "deserialize_yyyymmdd")]
    pub date: chrono::NaiveDate,

    /// Indicates whether service is available on the date specified in the date
    /// field.
    ///
    /// # Example
    ///
    /// Suppose a route has one set of trips available on holidays and another set of
    /// trips available on all other days. One `service_id` could correspond to the
    /// regular service schedule and another `service_id` could correspond to the
    /// holiday schedule. For a particular holiday, the calendar_dates.txt file could
    /// be used to add the holiday to the holiday `service_id` and to remove the
    /// holiday from the regular `service_id` schedule.
    pub exception_type: ExceptionType,
}
