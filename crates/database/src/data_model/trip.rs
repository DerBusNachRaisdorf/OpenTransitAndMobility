use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use model::{
    origin::{Origin, OriginalIdMapping},
    stop::Stop,
    trip::{StopTime, Trip},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::{Repo, Result, SubjectRepo, TripRepo};
use sqlx::prelude::FromRow;
use utility::id::{Id, IdWrapper};

use crate::{
    queries::trip::{
        delete_stop_times, exists, exists_with_origin, get, get_all,
        get_all_via_stop, get_stop_times, id_by_original_id, insert, put,
        put_original_id, put_stop_time, update,
    },
    PgDatabaseAutocommit, PgDatabaseTransaction,
};

use super::DatabaseRow;

#[derive(Debug, Clone, FromRow)]
pub struct TripRow {
    pub id: String,
    pub origin: String,
    pub line_id: String,
    pub service_id: Option<i32>,
    pub headsign: Option<String>,
    pub short_name: Option<String>,
}

impl DatabaseRow for TripRow {
    type Model = Trip;

    fn get_id(&self) -> Id<Self::Model> {
        Id::new(self.id.clone())
    }

    fn get_origin(&self) -> Id<Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> Trip {
        Trip {
            line_id: Id::new(self.line_id),
            service_id: self.service_id.map(Id::new),
            headsign: self.headsign,
            short_name: self.short_name,
            stops: vec![],
        }
    }

    fn from_model(trip: WithOrigin<Trip>) -> Self {
        Self {
            id: "".to_owned(),
            origin: trip.origin.raw(),
            line_id: trip.content.line_id.raw(),
            service_id: trip.content.service_id.raw(),
            headsign: trip.content.headsign,
            short_name: trip.content.short_name,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct StopTimeRow {
    pub origin: String,
    pub trip_id: String,
    pub stop_sequence: i32,
    pub stop_id: Option<String>,
    pub arrival_time: Option<i64>,
    pub departure_time: Option<i64>,
    pub stop_headsign: Option<String>,
}

impl StopTimeRow {
    pub fn to_model(self) -> StopTime {
        StopTime {
            stop_sequence: self.stop_sequence,
            stop_id: self.stop_id.map(Id::new),
            arrival_time: self.arrival_time.map(Duration::seconds),
            departure_time: self.departure_time.map(Duration::seconds),
            stop_headsign: self.stop_headsign,
        }
    }

    pub fn from_model(trip_id: Id<Trip>, stop_time: WithOrigin<StopTime>) -> Self {
        Self {
            origin: stop_time.origin.raw(),
            trip_id: trip_id.raw(),
            stop_sequence: stop_time.content.stop_sequence,
            stop_id: stop_time.content.stop_id.raw(),
            arrival_time: stop_time
                .content
                .arrival_time
                .map(|time| time.num_seconds()),
            departure_time: stop_time
                .content
                .departure_time
                .map(|time| time.num_seconds()),
            stop_headsign: stop_time.content.stop_headsign,
        }
    }
}

// Repo

#[async_trait]
impl Repo<Trip> for PgDatabaseAutocommit {
    async fn get(&mut self, id: Id<Trip>) -> Result<DatabaseEntry<Trip>> {
        get(&self.pool, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Trip>>> {
        get_all(&self.pool).await
    }

    async fn insert(
        &mut self,
        element: WithOrigin<Trip>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        insert(&self.pool, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Trip>>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        put(&self.pool, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Trip>>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        update(&self.pool, element).await
    }

    async fn exists(&mut self, id: Id<Trip>) -> Result<bool> {
        exists(&self.pool, id).await
    }

    async fn exists_with_origin(
        &mut self,
        id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<bool> {
        exists_with_origin(&self.pool, id, origin).await
    }
}

#[async_trait]
impl<'a> Repo<Trip> for PgDatabaseTransaction<'a> {
    async fn get(&mut self, id: Id<Trip>) -> Result<DatabaseEntry<Trip>> {
        get(&mut *self.tx, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Trip>>> {
        get_all(&mut *self.tx).await
    }

    async fn insert(
        &mut self,
        element: WithOrigin<Trip>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        insert(&mut *self.tx, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Trip>>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        put(&mut *self.tx, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Trip>>,
    ) -> Result<WithOrigin<WithId<Trip>>> {
        update(&mut *self.tx, element).await
    }

    async fn exists(&mut self, id: Id<Trip>) -> Result<bool> {
        exists(&mut *self.tx, id).await
    }

    async fn exists_with_origin(
        &mut self,
        id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<bool> {
        exists_with_origin(&mut *self.tx, id, origin).await
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<Trip> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Trip>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Trip>,
    ) -> Result<OriginalIdMapping<Trip>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<Trip> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Trip>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Trip>,
    ) -> Result<OriginalIdMapping<Trip>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}

// Trip Repo

#[async_trait]
impl TripRepo for PgDatabaseAutocommit {
    async fn put_stop_time(
        &mut self,
        trip_id: Id<Trip>,
        stop_time: WithOrigin<StopTime>,
    ) -> Result<WithOrigin<StopTime>> {
        put_stop_time(&self.pool, trip_id, stop_time).await
    }

    async fn get_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<Vec<StopTime>> {
        get_stop_times(&self.pool, trip_id, origin).await
    }

    async fn delete_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<()> {
        delete_stop_times(&self.pool, trip_id, origin).await
    }

    async fn get_all_via_stop(
        &mut self,
        stops: &[&Id<Stop>],
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<DatabaseEntry<Trip>>> {
        get_all_via_stop(&self.pool, stops, start, end).await
    }
}

#[async_trait]
impl<'a> TripRepo for PgDatabaseTransaction<'a> {
    async fn put_stop_time(
        &mut self,
        trip_id: Id<Trip>,
        stop_time: WithOrigin<StopTime>,
    ) -> Result<WithOrigin<StopTime>> {
        put_stop_time(&mut *self.tx, trip_id, stop_time).await
    }

    async fn get_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<Vec<StopTime>> {
        get_stop_times(&mut *self.tx, trip_id, origin).await
    }

    async fn delete_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<()> {
        delete_stop_times(&mut *self.tx, trip_id, origin).await
    }

    async fn get_all_via_stop(
        &mut self,
        stops: &[&Id<Stop>],
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<DatabaseEntry<Trip>>> {
        get_all_via_stop(&mut *self.tx, stops, start, end).await
    }
}
