use async_trait::async_trait;
use model::WithId;
use public_transport::{
    collector::{Collector, CollectorInstance},
    database::{CollectorRepo, Result},
};
use sqlx::{prelude::FromRow, types::Json};
use utility::id::Id;

use crate::{
    queries::collector::{get, get_all, set_state},
    PgDatabaseAutocommit, PgDatabaseTransaction,
};

#[derive(Debug, Clone, FromRow)]
pub struct CollectorRow<C: Collector> {
    pub id: i32,
    pub origin: String,
    pub kind: String,
    pub is_active: bool,
    pub state: Json<C::State>,
}

#[async_trait]
impl CollectorRepo for PgDatabaseAutocommit {
    async fn collectors<C>(&mut self) -> Result<Vec<WithId<CollectorInstance<C>>>>
    where
        C: Collector + 'static,
    {
        get_all(&self.pool).await
    }

    async fn get_collector<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
    ) -> Result<CollectorInstance<C>>
    where
        C: Collector + 'static,
    {
        get(&self.pool, id).await
    }

    async fn set_collector_state<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
        state: C::State,
    ) -> Result<C::State>
    where
        C: Collector + 'static,
    {
        set_state(&self.pool, id, state).await
    }
}

#[async_trait]
impl<'a> CollectorRepo for PgDatabaseTransaction<'a> {
    async fn collectors<C>(&mut self) -> Result<Vec<WithId<CollectorInstance<C>>>>
    where
        C: Collector + 'static,
    {
        get_all(&mut *self.tx).await
    }

    async fn get_collector<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
    ) -> Result<CollectorInstance<C>>
    where
        C: Collector + 'static,
    {
        get(&mut *self.tx, id).await
    }

    async fn set_collector_state<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
        state: C::State,
    ) -> Result<C::State>
    where
        C: Collector + 'static,
    {
        set_state(&mut *self.tx, id, state).await
    }
}
