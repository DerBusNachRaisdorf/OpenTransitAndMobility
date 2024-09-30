use model::WithId;
use public_transport::collector::{Collector, CollectorInstance};
use public_transport::database::Result;
use sqlx::types::Json;
use sqlx::{Executor, Postgres};
use utility::{id::Id, let_also::LetAlso};

use crate::data_model::collector::CollectorRow;

use super::convert_error;

pub async fn get_all<'c, E, C>(
    executor: E,
) -> Result<Vec<WithId<CollectorInstance<C>>>>
where
    E: Executor<'c, Database = Postgres>,
    C: Collector + 'static,
    <C as Collector>::State: 'static,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, kind, is_active, state
        FROM
            collectors
        WHERE
            kind = $1;
        ",
    )
    .bind(C::unique_id())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .into_iter()
    .map(|row: CollectorRow<C>| {
        WithId::new(
            Id::new(row.id),
            CollectorInstance {
                origin: Id::new(row.origin),
                is_active: row.is_active,
                state: row.state.0,
            },
        )
    })
    .collect::<Vec<_>>()
    .let_owned(Ok)
}

pub async fn get<'c, E, C>(
    executor: E,
    id: &Id<CollectorInstance<C>>,
) -> Result<CollectorInstance<C>>
where
    E: Executor<'c, Database = Postgres>,
    C: Collector + 'static,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, kind, is_active, state
        FROM
            collectors
        WHERE
            id = $1 AND kind = $2;
        ",
    )
    .bind(id.raw())
    .bind(C::unique_id())
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: CollectorRow<C>| CollectorInstance {
        origin: Id::new(row.origin),
        is_active: row.is_active,
        state: row.state.0,
    })
}

pub async fn set_state<'c, E, C>(
    executor: E,
    id: &Id<CollectorInstance<C>>,
    state: C::State,
) -> Result<C::State>
where
    E: Executor<'c, Database = Postgres>,
    C: Collector + 'static,
{
    sqlx::query_as(
        "
        UPDATE
            collectors
        SET
            state = $1
        WHERE
            id = $2 AND kind = $3
        RETURNING *;
        ",
    )
    .bind(Json(state))
    .bind(id.raw())
    .bind(C::unique_id()) // just for safety
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: CollectorRow<C>| row.state.0)
}
