use std::{env, error::Error, future::Future};

use async_trait::async_trait;
use model::{origin::Origin, WithId};
use public_transport::database::{
    Database, DatabaseAutocommit, DatabaseError, DatabaseOperations,
    DatabaseTransaction,
};
use queries::convert_error;
use sqlx::Transaction;

pub mod data_model;
pub mod queries;

pub struct DatabaseConnectionInfo {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub port: u16,
    pub database: String,
}

impl DatabaseConnectionInfo {
    pub fn from_env() -> Option<Self> {
        let username = env::var("DATABASE_USER").ok()?;
        let password = env::var("DATABASE_PASSWORD").ok()?;
        let hostname = env::var("DATABASE_HOST").ok()?;
        let port: u16 = env::var("DATABASE_PORT").ok()?.parse().ok()?;
        let database = env::var("DATABASE_NAME").ok()?;
        Some(Self {
            username,
            password,
            hostname,
            port,
            database,
        })
    }

    pub(self) fn postgres_url(self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.hostname, self.port, self.database
        )
    }
}

#[derive(Clone)]
pub struct PgDatabase {
    connection: sqlx::PgPool,
}

pub struct PgDatabaseTransaction<'a> {
    tx: Transaction<'a, sqlx::Postgres>,
}

#[async_trait]
impl<'a> DatabaseTransaction for PgDatabaseTransaction<'a> {
    async fn commit(self) -> public_transport::database::Result<()> {
        self.tx.commit().await.map_err(|why| match why {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            _ => DatabaseError::Other(Box::new(why)),
        })
    }
}

pub struct PgDatabaseAutocommit {
    pool: sqlx::PgPool,
}

impl DatabaseAutocommit for PgDatabaseAutocommit {}

impl PgDatabase {
    pub async fn connect(
        database_connection_info: DatabaseConnectionInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let url = database_connection_info.postgres_url();
        let pool = sqlx::postgres::PgPool::connect(&url).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { connection: pool })
    }
}

#[async_trait]
impl Database for PgDatabase {
    type Transaction = PgDatabaseTransaction<'static>;
    type Autocommit = PgDatabaseAutocommit;

    const BULK_INSERT_MAX: usize = 999;

    fn auto(&self) -> Self::Autocommit {
        PgDatabaseAutocommit {
            pool: self.connection.clone(),
        }
    }

    async fn transaction(
        &self,
    ) -> public_transport::database::Result<Self::Transaction> {
        let tx: Transaction<'_, sqlx::Postgres> = self
            .connection
            .begin()
            .await
            .map_err(|why| convert_error(why))?;

        Ok(PgDatabaseTransaction { tx })
    }

    async fn perform_transaction<T, F, Fut>(
        &self,
        action: F,
    ) -> public_transport::database::Result<T>
    where
        T: Send,
        F: Send + FnOnce(&mut Self::Transaction) -> Fut + Send,
        Fut: Future<Output = public_transport::database::Result<T>> + Send,
    {
        let tx: Transaction<'_, sqlx::Postgres> = self
            .connection
            .begin()
            .await
            .map_err(|why| convert_error(why))?;

        // run operations
        let mut tx = PgDatabaseTransaction { tx };
        let result = action(&mut tx).await;

        tx.commit().await?;

        result
    }
}

#[async_trait]
impl DatabaseOperations for PgDatabaseAutocommit {
    async fn origins(
        &mut self,
    ) -> public_transport::database::Result<Vec<WithId<Origin>>> {
        queries::origin::get_all(&self.pool).await
    }

    async fn put_origin(
        &mut self,
        origin: WithId<Origin>,
    ) -> public_transport::database::Result<WithId<Origin>> {
        queries::origin::put(&self.pool, origin).await
    }
}

#[async_trait]
impl<'a> DatabaseOperations for PgDatabaseTransaction<'a> {
    async fn origins(
        &mut self,
    ) -> public_transport::database::Result<Vec<WithId<Origin>>> {
        queries::origin::get_all(&mut *self.tx).await
    }

    async fn put_origin(
        &mut self,
        origin: WithId<Origin>,
    ) -> public_transport::database::Result<WithId<Origin>> {
        queries::origin::put(&mut *self.tx, origin).await
    }
}
