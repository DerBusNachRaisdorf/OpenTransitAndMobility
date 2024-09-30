use async_trait::async_trait;
use model::{
    origin::{Origin, OriginalIdMapping},
    shared_mobility::{RentalUris, SharedMobilityStation, Status},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::{Result, SharedMobilityStationRepo, SubjectRepo};
use sqlx::{prelude::FromRow, types::Json};
use utility::id::Id;

use crate::{
    queries::shared_mobility::{
        get_nearby, id_by_original_id, put_all, put_original_id, update_status,
    },
    PgDatabaseAutocommit, PgDatabaseTransaction,
};

use super::DatabaseRow;

#[derive(Debug, Clone, FromRow)]
pub struct SharedMobilityStationRow {
    pub id: String,
    pub origin: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub capacity: i32,
    pub rental_uri_android: Option<String>,
    pub rental_uri_ios: Option<String>,
    pub rental_uri_web: Option<String>,
    pub status: Option<Json<Status>>,
}

impl DatabaseRow for SharedMobilityStationRow {
    type Model = SharedMobilityStation;

    fn get_id(&self) -> utility::id::Id<Self::Model> {
        Id::new(self.id.clone())
    }

    fn get_origin(&self) -> Id<model::origin::Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> Self::Model {
        SharedMobilityStation {
            name: self.name,
            latitude: self.latitude,
            longitude: self.longitude,
            capacity: self.capacity as u32,
            rental_uris: RentalUris {
                android: self.rental_uri_android,
                ios: self.rental_uri_ios,
                web: self.rental_uri_web,
            },
            status: self.status.map(|s| s.0),
        }
    }

    fn from_model(_model: model::WithOrigin<Self::Model>) -> Self {
        todo!() // should be deprecated...
    }
}

// Repo

#[async_trait]
impl SharedMobilityStationRepo for PgDatabaseAutocommit {
    async fn find_nearby_shared_mobility_stations(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<SharedMobilityStation>>> {
        get_nearby(&self.pool, latitude, longitude, radius).await
    }

    async fn put_shared_mobility_stations(
        &mut self,
        origin: &Id<Origin>,
        stations: &[WithId<SharedMobilityStation>],
    ) -> Result<WithOrigin<Vec<WithId<SharedMobilityStation>>>> {
        put_all(&self.pool, origin, stations).await
    }

    async fn update_shared_mobility_station_status(
        &mut self,
        origin: &Id<Origin>,
        id: &Id<SharedMobilityStation>,
        status: Option<Status>,
    ) -> Result<()> {
        update_status(&self.pool, origin, id, status).await
    }
}

#[async_trait]
impl<'a> SharedMobilityStationRepo for PgDatabaseTransaction<'a> {
    async fn find_nearby_shared_mobility_stations(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<SharedMobilityStation>>> {
        get_nearby(&mut *self.tx, latitude, longitude, radius).await
    }

    async fn put_shared_mobility_stations(
        &mut self,
        origin: &Id<Origin>,
        stations: &[WithId<SharedMobilityStation>],
    ) -> Result<WithOrigin<Vec<WithId<SharedMobilityStation>>>> {
        put_all(&mut *self.tx, origin, stations).await
    }

    async fn update_shared_mobility_station_status(
        &mut self,
        origin: &Id<Origin>,
        id: &Id<SharedMobilityStation>,
        status: Option<Status>,
    ) -> Result<()> {
        update_status(&mut *self.tx, origin, id, status).await
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<SharedMobilityStation> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<SharedMobilityStation>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<SharedMobilityStation>,
    ) -> Result<OriginalIdMapping<SharedMobilityStation>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<SharedMobilityStation> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<SharedMobilityStation>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<SharedMobilityStation>,
    ) -> Result<OriginalIdMapping<SharedMobilityStation>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}
