use model::{
    agency::Agency,
    line::Line,
    origin::{Origin, OriginalIdMapping},
    stop::Stop,
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::Result;
use utility::{
    id::{Id, IdWrapper},
    let_also::LetAlso,
};

use crate::data_model::{
    line::{LineRow, RowLineType},
    with_origin_and_id, with_origins, with_origins_and_ids,
};
use sqlx::{Executor, Postgres};

use super::convert_error;

// Repo

pub async fn get<'c, E>(executor: E, id: Id<Line>) -> Result<DatabaseEntry<Line>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, kind, agency_id
        FROM lines
        WHERE id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|agencies: Vec<LineRow>| {
        Ok(DatabaseEntry::gather(id, with_origins(agencies)))
    })
}

pub async fn get_all<'c, E>(executor: E) -> Result<Vec<DatabaseEntry<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, kind, agency_id
        FROM lines;
        ",
    )
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|agencies: Vec<LineRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(agencies)))
    })
}

pub async fn insert<'c, E>(
    executor: E,
    line: WithOrigin<Line>,
) -> Result<WithOrigin<WithId<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO lines(
            origin,
            name,
            kind,
            agency_id
        )
        VALUES ($1, $2, $3, $4)
        RETURNING *;
        ",
    )
    .bind(line.origin.raw())
    .bind(line.content.name)
    .bind(RowLineType::from_line_type(line.content.kind))
    .bind(line.content.agency_id.raw())
    .fetch_one(executor)
    .await
    .map(|row: LineRow| with_origin_and_id(row))
    .map_err(|why| convert_error(why))
}

pub async fn put<'c, E>(
    executor: E,
    line: WithOrigin<WithId<Line>>,
) -> Result<WithOrigin<WithId<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO lines(
            id,
            origin,
            name,
            kind,
            agency_id
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id, origin)
        DO UPDATE SET
            name = EXCLUDED.name,
            kind = EXCLUDED.kind,
            agency_id = EXCLUDED.agency_id
        RETURNING *;
        ",
    )
    .bind(line.content.id.raw())
    .bind(line.origin.raw())
    .bind(line.content.content.name)
    .bind(RowLineType::from_line_type(line.content.content.kind))
    .bind(line.content.content.agency_id.raw())
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: LineRow| with_origin_and_id(row))
}

pub async fn update<'c, E>(
    executor: E,
    line: WithOrigin<WithId<Line>>,
) -> Result<WithOrigin<WithId<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        UPDATE lines
        SET name = $1,
            kind = $2,
            agency_id = $3
        WHERE origin = $4 AND id = $5
        RETURNING *;
        ",
    )
    .bind(line.content.content.name)
    .bind(RowLineType::from_line_type(line.content.content.kind))
    .bind(line.content.content.agency_id.raw())
    .bind(line.origin.raw())
    .bind(line.content.id.raw())
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: LineRow| with_origin_and_id(row))
}

pub async fn exists<'c, E>(executor: E, id: Id<Line>) -> Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: use sql
    get(executor, id).await.map(|entry| entry.contains_data())
}

pub async fn exists_with_origin<'c, E>(
    executor: E,
    id: Id<Line>,
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
) -> public_transport::database::Result<Option<Id<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::id_by_original_id(
        executor,
        origin,
        original_id,
        "lines_original_ids",
    )
    .await
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<Line>,
) -> public_transport::database::Result<OriginalIdMapping<Line>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::put_original_id(
        executor,
        origin,
        original_id,
        id,
        "lines_original_ids",
    )
    .await
}

// Line Repo

pub async fn get_by_name_and_agency<'c, E, N>(
    executor: E,
    name: N,
    agency: &Id<Agency>,
) -> Result<Vec<DatabaseEntry<Line>>>
where
    E: Executor<'c, Database = Postgres>,
    N: Into<String> + Send,
{
    sqlx::query_as(
        "
        SELECT id, origin, name, kind, agency_id
        FROM lines
        WHERE name = $1 AND agency_id = $2;
        ",
    )
    .bind(name.into())
    .bind(agency.raw())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|agencies: Vec<LineRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(agencies)))
    })
}

pub async fn get_by_stop_id<'c, E>(
    executor: E,
    stop_id: Id<Stop>,
) -> Result<Vec<DatabaseEntry<Line>>>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: take timespan? to avoid displaying old, discontinued lines
    sqlx::query_as(
        "
        SELECT DISTINCT
            l.id, l.origin, l.name, l.kind, l.agency_id
        FROM
            lines l
            JOIN trips t ON l.id = t.line_id
            JOIN stop_times st ON t.id = st.trip_id
        WHERE
            st.stop_id = $1;
        ",
    )
    .bind(stop_id.raw())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|agencies: Vec<LineRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(agencies)))
    })
}

pub async fn merge_candidates<'c, E>(
    executor: E,
    line: &Line,
    excluded_origin: &Id<Origin>,
) -> Result<Vec<WithOrigin<WithId<Line>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    let Some(name) = line.name.as_ref() else {
        return Ok(vec![]);
    };
    sqlx::query_as(
        "
        SELECT
            id, origin, name, kind, agency_id
        FROM
            lines
        WHERE
            name % $1
                AND NOT EXISTS (
                    SELECT 1 FROM lines s2
                    WHERE s2.id = lines.id
                    AND s2.origin = $2
                );
        ",
    )
    .bind(name)
    .bind(excluded_origin.raw_ref::<str>())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|agencies: Vec<LineRow>| Ok(with_origins_and_ids(agencies)))
}
