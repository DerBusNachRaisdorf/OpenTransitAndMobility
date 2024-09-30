use super::DatabaseRow;
use crate::{
    queries::stop::{
        exists, exists_with_origin, get, get_all, get_by_name, get_nearby,
        id_by_original_id, insert, merge_candidates, put, put_original_id, search,
        update,
    },
    PgDatabaseAutocommit, PgDatabaseTransaction,
};
use async_trait::async_trait;
use model::{
    origin::{Origin, OriginalIdMapping},
    stop::{Location, Stop},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::{MergableRepo, Repo, Result, StopRepo, SubjectRepo};
use sqlx::prelude::FromRow;
use utility::id::{Id, IdWrapper};

#[derive(Debug, Clone, FromRow)]
pub struct StopRow {
    pub id: String,
    pub origin: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub platform_code: Option<String>,
}

impl DatabaseRow for StopRow {
    type Model = Stop;

    fn get_id(&self) -> Id<Self::Model> {
        Id::new(self.id.clone())
    }

    fn get_origin(&self) -> Id<Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> Self::Model {
        Stop {
            name: self.name,
            description: self.description,
            parent_id: self.parent_id.map(|id| Id::new(id)),
            location: match (self.latitude, self.longitude) {
                (Some(lat), Some(long)) => Some(Location {
                    latitude: lat,
                    longitude: long,
                    address: self.address,
                }),
                _ => None,
            },
            platform_code: self.platform_code,
        }
    }

    fn from_model(stop: WithOrigin<Self::Model>) -> Self {
        Self {
            id: "".to_owned(),
            origin: stop.origin.raw(),
            name: stop.content.name,
            description: stop.content.description,
            parent_id: stop.content.parent_id.raw(),
            latitude: stop
                .content
                .location
                .as_ref()
                .map(|location| location.latitude),
            longitude: stop
                .content
                .location
                .as_ref()
                .map(|location| location.longitude),
            address: stop.content.location.and_then(|location| location.address),
            platform_code: stop.content.platform_code,
        }
    }
}

// Repo

#[async_trait]
impl Repo<Stop> for PgDatabaseAutocommit {
    async fn get(&mut self, id: Id<Stop>) -> Result<DatabaseEntry<Stop>> {
        get(&self.pool, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_all(&self.pool).await
    }

    async fn insert(
        &mut self,
        element: WithOrigin<Stop>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        insert(&self.pool, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Stop>>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        put(&self.pool, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Stop>>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        update(&self.pool, element).await
    }

    async fn exists(&mut self, id: Id<Stop>) -> Result<bool> {
        exists(&self.pool, id).await
    }

    async fn exists_with_origin(
        &mut self,
        id: Id<Stop>,
        origin: Id<Origin>,
    ) -> Result<bool> {
        exists_with_origin(&self.pool, id, origin).await
    }
}

#[async_trait]
impl<'a> Repo<Stop> for PgDatabaseTransaction<'a> {
    async fn get(&mut self, id: Id<Stop>) -> Result<DatabaseEntry<Stop>> {
        get(&mut *self.tx, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_all(&mut *self.tx).await
    }

    async fn insert(
        &mut self,
        element: WithOrigin<Stop>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        insert(&mut *self.tx, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Stop>>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        put(&mut *self.tx, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Stop>>,
    ) -> Result<WithOrigin<WithId<Stop>>> {
        update(&mut *self.tx, element).await
    }

    async fn exists(&mut self, id: Id<Stop>) -> Result<bool> {
        exists(&mut *self.tx, id).await
    }

    async fn exists_with_origin(
        &mut self,
        id: Id<Stop>,
        origin: Id<Origin>,
    ) -> Result<bool> {
        exists_with_origin(&mut *self.tx, id, origin).await
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<Stop> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Stop>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Stop>,
    ) -> Result<OriginalIdMapping<Stop>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<Stop> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Stop>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Stop>,
    ) -> Result<OriginalIdMapping<Stop>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}

// Stop Repo

#[async_trait]
impl StopRepo for PgDatabaseAutocommit {
    async fn find_nearby(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_nearby(&self.pool, latitude, longitude, radius).await
    }

    async fn stop_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_by_name(&self.pool, name).await
    }

    async fn search<S: Into<String> + Send>(
        &mut self,
        pattern: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        search(&self.pool, pattern).await
    }
}

#[async_trait]
impl<'a> StopRepo for PgDatabaseTransaction<'a> {
    async fn find_nearby(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_nearby(&mut *self.tx, latitude, longitude, radius).await
    }

    async fn stop_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        get_by_name(&mut *self.tx, name).await
    }

    async fn search<S: Into<String> + Send>(
        &mut self,
        pattern: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>> {
        search(&mut *self.tx, pattern).await
    }
}

// Mergable Repo

#[async_trait]
impl<'a> MergableRepo<Stop> for PgDatabaseTransaction<'a> {
    async fn merge_candidates(
        &mut self,
        element: &Stop,
        excluded_origin: &Id<Origin>,
    ) -> Result<Vec<WithOrigin<WithId<Stop>>>> {
        merge_candidates(&mut *self.tx, element, excluded_origin).await
    }
}

#[async_trait]
impl MergableRepo<Stop> for PgDatabaseAutocommit {
    async fn merge_candidates(
        &mut self,
        element: &Stop,
        excluded_origin: &Id<Origin>,
    ) -> Result<Vec<WithOrigin<WithId<Stop>>>> {
        merge_candidates(&self.pool, element, excluded_origin).await
    }
}
