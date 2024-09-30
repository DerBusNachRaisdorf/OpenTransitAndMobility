use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate};
use model::origin::Origin;
use model::trip::Trip;
use model::trip_update::{StopTimeUpdate, TripUpdate, TripUpdateId};
use model::{DatabaseEntry, DateTimeRange, WithId, WithOrigin};
use public_transport::database::{RealtimeRepo, Result};
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use utility::id::Id;

use crate::queries::trip_update::{
    get, get_for_trips_in_range, get_timestamp, put_all,
};
use crate::{PgDatabaseAutocommit, PgDatabaseTransaction};

use super::DatabaseRow;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "trip_status", rename_all = "snake_case")]
pub enum TripStatus {
    Scheduled,
    Unscheduled,
    Cancelled,
    Added,
    Deleted,
}

impl Into<model::trip_update::TripStatus> for TripStatus {
    fn into(self) -> model::trip_update::TripStatus {
        match self {
            Self::Scheduled => model::trip_update::TripStatus::Scheduled,
            Self::Unscheduled => model::trip_update::TripStatus::Unscheduled,
            Self::Cancelled => model::trip_update::TripStatus::Cancelled,
            Self::Added => model::trip_update::TripStatus::Added,
            Self::Deleted => model::trip_update::TripStatus::Deleted,
        }
    }
}

impl From<model::trip_update::TripStatus> for TripStatus {
    fn from(value: model::trip_update::TripStatus) -> Self {
        match value {
            model::trip_update::TripStatus::Scheduled => Self::Scheduled,
            model::trip_update::TripStatus::Unscheduled => Self::Unscheduled,
            model::trip_update::TripStatus::Cancelled => Self::Cancelled,
            model::trip_update::TripStatus::Added => Self::Added,
            model::trip_update::TripStatus::Deleted => Self::Deleted,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TripUpdateRow {
    pub origin: String,
    pub trip_id: String,
    pub trip_start_date: NaiveDate,
    pub status: TripStatus,
    pub stop_time_updates: Json<Vec<StopTimeUpdate>>,
    pub timestamp: Option<DateTime<Local>>,
}

impl DatabaseRow for TripUpdateRow {
    type Model = TripUpdate;

    fn get_id(&self) -> Id<Self::Model> {
        Id::new(TripUpdateId::new(
            Id::new(self.trip_id.clone()),
            self.trip_start_date.clone(),
        ))
    }

    fn get_origin(&self) -> Id<Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> TripUpdate {
        TripUpdate {
            status: self.status.into(),
            stops: self.stop_time_updates.0,
            timestamp: self.timestamp,
        }
    }

    fn from_model(_update: WithOrigin<TripUpdate>) -> Self {
        todo!() // diese methode sollte depricated sein.
    }
}

#[async_trait]
impl RealtimeRepo for PgDatabaseAutocommit {
    async fn put_trip_updates(
        &mut self,
        origin: &Id<Origin>,
        updates: &[WithId<TripUpdate>],
    ) -> Result<WithOrigin<Vec<WithId<TripUpdate>>>> {
        put_all(&self.pool, origin, updates).await
    }

    async fn get_realtime_for_trip(
        &mut self,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<DatabaseEntry<TripUpdate>> {
        get(&self.pool, trip_id, trip_start_date).await
    }

    async fn get_timestamp(
        &mut self,
        origin: &Id<Origin>,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<Option<DateTime<Local>>> {
        get_timestamp(&self.pool, origin, trip_id, trip_start_date).await
    }

    async fn get_realtime_for_trips_in_range<'c>(
        &mut self,
        trip_ids: &[Id<Trip>],
        range: DateTimeRange<Local>,
    ) -> Result<Vec<DatabaseEntry<TripUpdate>>> {
        get_for_trips_in_range(&self.pool, trip_ids, range).await
    }
}

#[async_trait]
impl<'a> RealtimeRepo for PgDatabaseTransaction<'a> {
    async fn put_trip_updates(
        &mut self,
        origin: &Id<Origin>,
        updates: &[WithId<TripUpdate>],
    ) -> Result<WithOrigin<Vec<WithId<TripUpdate>>>> {
        put_all(&mut *self.tx, origin, updates).await
    }

    async fn get_realtime_for_trip(
        &mut self,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<DatabaseEntry<TripUpdate>> {
        get(&mut *self.tx, trip_id, trip_start_date).await
    }

    async fn get_timestamp(
        &mut self,
        origin: &Id<Origin>,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<Option<DateTime<Local>>> {
        get_timestamp(&mut *self.tx, origin, trip_id, trip_start_date).await
    }

    async fn get_realtime_for_trips_in_range<'c>(
        &mut self,
        trip_ids: &[Id<Trip>],
        range: DateTimeRange<Local>,
    ) -> Result<Vec<DatabaseEntry<TripUpdate>>> {
        get_for_trips_in_range(&mut *self.tx, trip_ids, range).await
    }
}
