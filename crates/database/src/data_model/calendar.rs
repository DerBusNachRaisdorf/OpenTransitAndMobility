use async_trait::async_trait;
use model::{
    calendar::{CalendarDate, CalendarWindow, Service},
    origin::{Origin, OriginalIdMapping},
};
use public_transport::database::{self, ServiceRepo, SubjectRepo};
use sqlx::prelude::FromRow;
use utility::id::Id;

use crate::{
    queries::service::{
        get_calendar_dates, get_calendar_windows, id_by_original_id,
        put_calendar_date, put_calendar_window, put_original_id,
    },
    PgDatabaseAutocommit, PgDatabaseTransaction,
};

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "service_availability", rename_all = "snake_case")]
pub enum ServiceAvailability {
    Available,
    Unavailable,
}

impl Into<model::calendar::ServiceAvailability> for ServiceAvailability {
    fn into(self) -> model::calendar::ServiceAvailability {
        match self {
            Self::Available => model::calendar::ServiceAvailability::Available,
            Self::Unavailable => model::calendar::ServiceAvailability::Unavailable,
        }
    }
}

impl From<model::calendar::ServiceAvailability> for ServiceAvailability {
    fn from(value: model::calendar::ServiceAvailability) -> Self {
        match value {
            model::calendar::ServiceAvailability::Available => Self::Available,
            model::calendar::ServiceAvailability::Unavailable => Self::Unavailable,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct CalendarWindowRow {
    pub service_id: Option<i32>,
    pub monday: ServiceAvailability,
    pub tuesday: ServiceAvailability,
    pub wednesday: ServiceAvailability,
    pub thursday: ServiceAvailability,
    pub friday: ServiceAvailability,
    pub saturday: ServiceAvailability,
    pub sunday: ServiceAvailability,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
}

impl CalendarWindowRow {
    pub fn to_model(self) -> CalendarWindow {
        CalendarWindow {
            monday: self.monday.into(),
            tuesday: self.tuesday.into(),
            wednesday: self.wednesday.into(),
            thursday: self.thursday.into(),
            friday: self.friday.into(),
            saturday: self.saturday.into(),
            sunday: self.sunday.into(),
            start_date: self.start_date,
            end_date: self.end_date,
        }
    }

    pub fn from_model(calendar: CalendarWindow) -> Self {
        Self {
            service_id: None,
            monday: calendar.monday.into(),
            tuesday: calendar.tuesday.into(),
            wednesday: calendar.wednesday.into(),
            thursday: calendar.thursday.into(),
            friday: calendar.friday.into(),
            saturday: calendar.saturday.into(),
            sunday: calendar.sunday.into(),
            start_date: calendar.start_date,
            end_date: calendar.end_date,
        }
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<Service> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> database::Result<Option<Id<Service>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Service>,
    ) -> database::Result<OriginalIdMapping<Service>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<Service> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> database::Result<Option<Id<Service>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Service>,
    ) -> database::Result<OriginalIdMapping<Service>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}

// Service Repo

#[async_trait]
impl ServiceRepo for PgDatabaseAutocommit {
    async fn put_calendar_window(
        &mut self,
        service_id: Option<&Id<Service>>,
        window: CalendarWindow,
    ) -> database::Result<(Id<Service>, CalendarWindow)> {
        put_calendar_window(&self.pool, service_id.cloned(), window).await
    }

    async fn put_calendar_date(
        &mut self,
        service_id: Option<&Id<Service>>,
        date: CalendarDate,
    ) -> database::Result<(Id<Service>, CalendarDate)> {
        put_calendar_date(&self.pool, service_id.cloned(), date).await
    }

    async fn get_calendar_windows(
        &mut self,
        service_id: &Id<Service>,
    ) -> database::Result<Vec<CalendarWindow>> {
        get_calendar_windows(&self.pool, service_id).await
    }

    async fn get_calendar_dates(
        &mut self,
        service_id: &Id<Service>,
    ) -> database::Result<Vec<CalendarDate>> {
        get_calendar_dates(&self.pool, service_id).await
    }
}

#[async_trait]
impl<'a> ServiceRepo for PgDatabaseTransaction<'a> {
    async fn put_calendar_window(
        &mut self,
        service_id: Option<&Id<Service>>,
        window: CalendarWindow,
    ) -> database::Result<(Id<Service>, CalendarWindow)> {
        put_calendar_window(&mut *self.tx, service_id.cloned(), window).await
    }

    async fn put_calendar_date(
        &mut self,
        service_id: Option<&Id<Service>>,
        date: CalendarDate,
    ) -> database::Result<(Id<Service>, CalendarDate)> {
        put_calendar_date(&mut *self.tx, service_id.cloned(), date).await
    }

    async fn get_calendar_windows(
        &mut self,
        service_id: &Id<Service>,
    ) -> database::Result<Vec<CalendarWindow>> {
        get_calendar_windows(&mut *self.tx, service_id).await
    }

    async fn get_calendar_dates(
        &mut self,
        service_id: &Id<Service>,
    ) -> database::Result<Vec<CalendarDate>> {
        get_calendar_dates(&mut *self.tx, service_id).await
    }
}
