use async_trait::async_trait;
use model::origin::OriginalIdMapping;
use model::{agency::Agency, origin::Origin, DatabaseEntry, WithId, WithOrigin};
use public_transport::database::{AgencyRepo, Repo, Result, SubjectRepo};
use sqlx::prelude::FromRow;
use utility::id::Id;

use crate::queries::agency::{
    exists, exists_with_origin, get, get_all, get_by_name, id_by_original_id, insert, put,
    put_original_id, update,
};
use crate::PgDatabaseAutocommit;
use crate::PgDatabaseTransaction;

use super::DatabaseRow;

#[derive(Debug, Clone, FromRow)]
pub struct AgencyRow {
    pub id: String,
    pub origin: String,
    pub name: String,
    pub website: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub fare_url: Option<String>,
}

// remove this completely
impl AgencyRow {
    // TODO: remove option
    pub fn from_model(agency: WithOrigin<Agency>) -> Option<Self> {
        Some(Self {
            id: "".to_owned(), // TODO! check in sql for empty string and not just null
            origin: agency.origin.to_string(),
            name: agency.content.name,
            website: agency.content.website,
            phone_number: agency.content.phone_number,
            email: agency.content.email,
            fare_url: agency.content.fare_url,
        })
    }
}

impl DatabaseRow for AgencyRow {
    type Model = Agency;

    fn get_id(&self) -> Id<Self::Model> {
        Id::new(self.id.clone())
    }

    fn get_origin(&self) -> Id<Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> Self::Model {
        Agency {
            name: self.name,
            website: self.website,
            phone_number: self.phone_number,
            email: self.email,
            fare_url: self.fare_url,
        }
    }

    fn from_model(agency: WithOrigin<Agency>) -> Self {
        Self {
            id: "".to_owned(), // TODO! check in sql for empty string and not just null
            origin: agency.origin.to_string(),
            name: agency.content.name,
            website: agency.content.website,
            phone_number: agency.content.phone_number,
            email: agency.content.email,
            fare_url: agency.content.fare_url,
        }
    }
}

// Repo

#[async_trait]
impl Repo<Agency> for PgDatabaseAutocommit {
    async fn get(&mut self, id: Id<Agency>) -> Result<DatabaseEntry<Agency>> {
        get(&self.pool, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Agency>>> {
        get_all(&self.pool).await
    }

    async fn insert(&mut self, element: WithOrigin<Agency>) -> Result<WithOrigin<WithId<Agency>>> {
        insert(&self.pool, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Agency>>,
    ) -> Result<WithOrigin<WithId<Agency>>> {
        put(&self.pool, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Agency>>,
    ) -> Result<WithOrigin<WithId<Agency>>> {
        update(&self.pool, element).await
    }

    async fn exists(&mut self, id: Id<Agency>) -> Result<bool> {
        exists(&self.pool, id).await
    }

    async fn exists_with_origin(&mut self, id: Id<Agency>, origin: Id<Origin>) -> Result<bool> {
        exists_with_origin(&self.pool, id, origin).await
    }
}

#[async_trait]
impl<'a> Repo<Agency> for PgDatabaseTransaction<'a> {
    async fn get(&mut self, id: Id<Agency>) -> Result<DatabaseEntry<Agency>> {
        get(&mut *self.tx, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Agency>>> {
        get_all(&mut *self.tx).await
    }

    async fn insert(&mut self, element: WithOrigin<Agency>) -> Result<WithOrigin<WithId<Agency>>> {
        insert(&mut *self.tx, element).await
    }

    async fn put(
        &mut self,
        element: WithOrigin<WithId<Agency>>,
    ) -> Result<WithOrigin<WithId<Agency>>> {
        put(&mut *self.tx, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Agency>>,
    ) -> Result<WithOrigin<WithId<Agency>>> {
        update(&mut *self.tx, element).await
    }

    async fn exists(&mut self, id: Id<Agency>) -> Result<bool> {
        exists(&mut *self.tx, id).await
    }

    async fn exists_with_origin(&mut self, id: Id<Agency>, origin: Id<Origin>) -> Result<bool> {
        exists_with_origin(&mut *self.tx, id, origin).await
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<Agency> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Agency>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Agency>,
    ) -> Result<OriginalIdMapping<Agency>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<Agency> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Agency>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Agency>,
    ) -> Result<OriginalIdMapping<Agency>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}

// Agency Repo

#[async_trait]
impl AgencyRepo for PgDatabaseAutocommit {
    async fn agency_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Agency>>> {
        get_by_name(&self.pool, name).await
    }
}

#[async_trait]
impl<'a> AgencyRepo for PgDatabaseTransaction<'a> {
    async fn agency_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Agency>>> {
        get_by_name(&mut *self.tx, name).await
    }
}
