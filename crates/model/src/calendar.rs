use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};
use schemars::JsonSchema;
use serde::Serialize;
use utility::id::HasId;

// TODO: rename file to 'service.rs'

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub enum ServiceAvailability {
    Available,
    Unavailable,
}

impl ServiceAvailability {
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::Available
        } else {
            Self::Unavailable
        }
    }

    pub fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }

    pub fn or(self, other: ServiceAvailability) -> ServiceAvailability {
        if self.is_available() {
            self
        } else {
            other
        }
    }

    pub fn or_else<F>(self, other: F) -> ServiceAvailability
    where
        F: FnOnce() -> ServiceAvailability,
    {
        if self.is_available() {
            self
        } else {
            other()
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Service {
    pub windows: Vec<CalendarWindow>,
    pub dates: Vec<CalendarDate>,
}

impl Service {
    pub fn check_availability(&self, date: chrono::NaiveDate) -> ServiceAvailability {
        ServiceAvailability::from_bool(
            self.windows
                .iter()
                .any(|entry| entry.check_availability(date).is_available())
                || self.dates.iter().any(|entry| {
                    entry.date == date
                        && entry.exception_type == ServiceExceptionType::Added
                }),
        )
    }

    /// Returns a sorted vec of all days, at which the service is available within
    /// an optionally specified range.
    pub fn available_days(
        &self,
        earliest: Option<NaiveDate>,
        latest: Option<NaiveDate>,
    ) -> Vec<NaiveDate> {
        // get all days in range from calendar windows
        let mut days = self
            .windows
            .iter()
            .flat_map(|window| window.available_days(earliest, latest))
            .collect::<HashSet<_>>();

        // add all days in range from calendar dates
        for date in self.dates.iter() {
            let is_in_range = match (earliest, latest) {
                (Some(earliest), Some(latest)) => {
                    date.date >= earliest && date.date <= latest
                }
                (Some(earliest), None) => date.date >= earliest,
                (None, Some(latest)) => date.date <= latest,
                _ => true,
            };

            if !is_in_range {
                continue;
            }

            if date.exception_type == ServiceExceptionType::Added {
                days.insert(date.date.clone());
            } else {
                days.remove(&date.date);
            }
        }

        // obtain all unique days and sort them
        let mut days = days.into_iter().collect::<Vec<_>>();
        days.sort();
        days
    }

    // moves all weekly repeating dates into a window and merges all windows with
    // the same start / end date.
    pub fn optimize(&mut self) {
        todo!()
    }
}

impl HasId for Service {
    type IdType = i32; // TODO: maybe bigger int?
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CalendarWindow {
    pub monday: ServiceAvailability,
    pub tuesday: ServiceAvailability,
    pub wednesday: ServiceAvailability,
    pub thursday: ServiceAvailability,
    pub friday: ServiceAvailability,
    pub saturday: ServiceAvailability,
    pub sunday: ServiceAvailability,
    #[serde(skip)] // TODO!
    pub start_date: chrono::NaiveDate,
    #[serde(skip)] // TODO!
    pub end_date: chrono::NaiveDate,
}

impl CalendarWindow {
    pub fn check_availability(&self, date: chrono::NaiveDate) -> ServiceAvailability {
        // service is not available, if date is not in range...
        if date < self.start_date || date > self.end_date {
            return ServiceAvailability::Unavailable;
        }
        // ...otherwise, the service is available, if it is availble on the according
        // day of week.
        match date.weekday() {
            Weekday::Mon => self.monday,
            Weekday::Tue => self.tuesday,
            Weekday::Wed => self.wednesday,
            Weekday::Thu => self.thursday,
            Weekday::Fri => self.friday,
            Weekday::Sat => self.saturday,
            Weekday::Sun => self.sunday,
        }
    }

    /// Returns a sorted vec of all days, at which the service is available
    /// within an optionally specified range.
    pub fn available_days(
        &self,
        earliest: Option<NaiveDate>,
        latest: Option<NaiveDate>,
    ) -> Vec<NaiveDate> {
        let start = earliest
            .filter(|earliest| *earliest > self.start_date)
            .unwrap_or(self.start_date);
        let end = latest
            .filter(|latest| *latest < self.end_date)
            .unwrap_or(self.end_date);

        let mut days = vec![];
        for day in start.iter_days() {
            if day > end {
                break;
            }
            if self.check_availability(day).is_available() {
                days.push(day);
            }
        }
        days
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub enum ServiceExceptionType {
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CalendarDate {
    #[serde(skip)] // TODO!
    pub date: chrono::NaiveDate,
    pub exception_type: ServiceExceptionType,
}
