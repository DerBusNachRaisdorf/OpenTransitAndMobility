use model::{
    agency::Agency,
    origin::{Origin, OriginalIdMapping},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::Result;
use utility::{id::Id, let_also::LetAlso};

use crate::data_model::{
    agency::AgencyRow, with_origin_and_id, with_origins, with_origins_and_ids,
};
use sqlx::{Executor, Postgres};

use super::convert_error;

pub trait SqlxExecutor<'c>: Executor<'c, Database = Postgres> {}

// Repo

pub async fn get<'c, E>(executor: E, id: Id<Agency>) -> Result<DatabaseEntry<Agency>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, website, phone_number, email, fare_url
        FROM agencies
        WHERE id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|agencies: Vec<AgencyRow>| {
        Ok(DatabaseEntry::gather(id, with_origins(agencies)))
    })
}

pub async fn get_all<'c, E>(executor: E) -> Result<Vec<DatabaseEntry<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, website, phone_number, email, fare_url
        FROM agencies;
        ",
    )
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|agencies: Vec<AgencyRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(agencies)))
    })
}

pub async fn insert<'c, E>(
    executor: E,
    agency: WithOrigin<Agency>,
) -> Result<WithOrigin<WithId<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO agencies(
            origin,
            name,
            website,
            phone_number,
            email,
            fare_url
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *;
        ",
    )
    .bind(agency.origin.raw())
    .bind(&agency.content.name)
    .bind(&agency.content.website)
    .bind(&agency.content.phone_number)
    .bind(&agency.content.email)
    .bind(&agency.content.fare_url)
    .fetch_one(executor)
    .await
    .map(|row: AgencyRow| with_origin_and_id(row))
    .map_err(|why| convert_error(why))
}

pub async fn put<'c, E>(
    executor: E,
    agency: WithOrigin<WithId<Agency>>,
) -> Result<WithOrigin<WithId<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO agencies(
            id,
            origin,
            name,
            website,
            phone_number,
            email,
            fare_url
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id, origin)
        DO UPDATE SET
            name = EXCLUDED.name,
            website = EXCLUDED.website,
            phone_number = EXCLUDED.phone_number,
            email = EXCLUDED.email,
            fare_url = EXCLUDED.fare_url
        RETURNING *;
        ",
    )
    .bind(agency.content.id.raw())
    .bind(agency.origin.raw())
    .bind(&agency.content.content.name)
    .bind(&agency.content.content.website)
    .bind(&agency.content.content.phone_number)
    .bind(&agency.content.content.email)
    .bind(&agency.content.content.fare_url)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: AgencyRow| with_origin_and_id(row))
}

pub async fn update<'c, E>(
    executor: E,
    agency: WithOrigin<WithId<Agency>>,
) -> Result<WithOrigin<WithId<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        UPDATE agencies
        SET name = $1,
            website = $2,
            phone_number = $3,
            email = $4,
            fare_url = $5
        WHERE origin = $6 AND id = $7
        RETURNING *;
        ",
    )
    .bind(&agency.content.content.name)
    .bind(&agency.content.content.website)
    .bind(&agency.content.content.phone_number)
    .bind(&agency.content.content.email)
    .bind(&agency.content.content.fare_url)
    .bind(agency.origin.raw())
    .bind(agency.content.id.raw())
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: AgencyRow| with_origin_and_id(row))
}

pub async fn exists<'c, E>(executor: E, id: Id<Agency>) -> Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: use sql
    get(executor, id).await.map(|entry| entry.contains_data())
}

pub async fn exists_with_origin<'c, E>(
    executor: E,
    id: Id<Agency>,
    origin: Id<Origin>,
) -> Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: use sql
    get(executor, id).await.map(|entry| {
        entry
            .source_data
            .into_iter()
            .any(|agency| agency.origin == origin)
    })
}

// Subject Repo

pub async fn id_by_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
) -> public_transport::database::Result<Option<Id<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::id_by_original_id(
        executor,
        origin,
        original_id,
        "agencies_original_ids",
    )
    .await
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<Agency>,
) -> public_transport::database::Result<OriginalIdMapping<Agency>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::put_original_id(
        executor,
        origin,
        original_id,
        id,
        "agencies_original_ids",
    )
    .await
}

// Agency Repo

pub async fn get_by_name<'c, E, S>(
    executor: E,
    name: S,
) -> Result<Vec<DatabaseEntry<Agency>>>
where
    E: Executor<'c, Database = Postgres>,
    S: Into<String> + Send,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, website, phone_number, email, fare_url
        FROM agencies WHERE name = $1;
        ",
    )
    .bind(name.into())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|agencies: Vec<AgencyRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(agencies)))
    })
}

pub async fn merge_candidates<'c, E>(
    executor: E,
    agency: &Agency,
    excluded_origin: &Id<Origin>,
) -> Result<Vec<WithOrigin<WithId<Agency>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, name, website, phone_number, email, fare_url
        FROM
            agencies
        WHERE
            name % $1
                AND NOT EXISTS (
                    SELECT 1 FROM agencies s2
                    WHERE s2.id = agencies.id
                    AND s2.origin = $2
                );
        ",
    )
    .bind(agency.name.as_str())
    .bind(excluded_origin.raw_ref::<str>())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|agencies: Vec<AgencyRow>| Ok(with_origins_and_ids(agencies)))
}
