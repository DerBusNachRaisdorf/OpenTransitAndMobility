use std::fmt::Debug;

use model::{
    origin::{Origin, OriginalIdMapping},
    WithId,
};
use public_transport::database::DatabaseError;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use utility::{
    id::{HasId, Id},
    let_also::LetAlso,
};

use crate::data_model::origin::{OriginRow, OriginalIdMappingRow};

use super::convert_error;

pub async fn get_all<'c, E>(
    executor: E,
) -> public_transport::database::Result<Vec<WithId<Origin>>>
where
    E: Executor<'c, Database = Postgres>,
{
    let results: Vec<OriginRow> =
        sqlx::query_as("SELECT * FROM origins ORDER BY priority ASC;")
            .fetch_all(executor)
            .await
            .map_err(|why| DatabaseError::Other(Box::new(why)))?;
    results
        .into_iter()
        .map(|row| {
            WithId::new(
                Id::new(row.id),
                Origin {
                    name: row.name,
                    priority: row.priority,
                },
            )
        })
        .collect::<Vec<_>>()
        .let_owned(|origins| Ok(origins))
}

pub async fn put<'c, E>(
    executor: E,
    origin: WithId<Origin>,
) -> public_transport::database::Result<WithId<Origin>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO origins(
            id,
            name,
            priority
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (id)
        DO UPDATE SET
            name = EXCLUDED.name,
            priority = EXCLUDED.priority
        RETURNING *;
        ",
    )
    .bind(origin.id.raw())
    .bind(origin.content.name)
    .bind(origin.content.priority)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: OriginRow| {
        WithId::new(
            Id::new(row.id),
            Origin {
                name: row.name,
                priority: row.priority,
            },
        )
    })
}

// id mapping

pub(crate) async fn id_by_original_id<'c, E, S>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    table_name: &str,
) -> public_transport::database::Result<Option<Id<S>>>
where
    E: Executor<'c, Database = Postgres>,
    S: HasId,
    S::IdType: Debug + Clone + Serialize + From<String>,
{
    sqlx::query_scalar(
        format!(
            "
        SELECT
            id
        FROM
            {}
        WHERE
            origin = $1 AND original_id = $2;
        ",
            table_name
        )
        .as_ref(),
    )
    .bind(origin.raw())
    .bind(original_id)
    .fetch_optional(executor)
    .await
    .map_err(convert_error)?
    .map(|id: String| Id::new(id.into()))
    .let_owned(|result| Ok(result))
}

pub(crate) async fn put_original_id<'c, E, S>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<S>,
    table_name: &str,
) -> public_transport::database::Result<OriginalIdMapping<S>>
where
    E: Executor<'c, Database = Postgres>,
    S: HasId,
    S::IdType: Debug + Clone + Serialize + From<String> + Into<String>,
{
    sqlx::query_as(
        format!(
            "
            INSERT INTO {}(
                origin,
                original_id,
                id
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (origin, original_id)
            DO UPDATE SET
                id = EXCLUDED.id
            RETURNING *;
            ",
            table_name
        )
        .as_ref(),
    )
    .bind(origin.raw())
    .bind(original_id)
    .bind(id.raw().into())
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: OriginalIdMappingRow<String>| row.to_model())
}
