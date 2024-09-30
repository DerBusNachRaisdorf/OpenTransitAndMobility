use chrono::{DateTime, Local};
use model::{
    origin::{Origin, OriginalIdMapping},
    stop::Stop,
    trip::{StopTime, Trip},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::Result;
use utility::{
    id::{Id, IdWrapper},
    let_also::LetAlso,
};

use crate::data_model::{
    trip::{StopTimeRow, TripRow},
    with_origin_and_id, with_origins, with_origins_and_ids,
};
use sqlx::{Executor, Postgres};

use super::convert_error;

// Repo

pub async fn get<'c, E>(executor: E, id: Id<Trip>) -> Result<DatabaseEntry<Trip>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, line_id, service_id, headsign, short_name
        FROM
            trips
        WHERE
            id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|trips: Vec<TripRow>| {
        Ok(DatabaseEntry::gather(id, with_origins(trips)))
    })
}

pub async fn get_all<'c, E>(executor: E) -> Result<Vec<DatabaseEntry<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, line_id, service_id, headsign, short_name
        FROM
            trips;
        ",
    )
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|trips: Vec<TripRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(trips)))
    })
}

pub async fn insert<'c, E>(
    executor: E,
    line: WithOrigin<Trip>,
) -> Result<WithOrigin<WithId<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO trips(
            origin,
            line_id,
            service_id,
            headsign,
            short_name
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *;
        ",
    )
    .bind(line.origin.raw())
    .bind(line.content.line_id.raw())
    .bind(line.content.service_id.raw())
    .bind(line.content.headsign)
    .bind(line.content.short_name)
    .fetch_one(executor)
    .await
    .map(|row: TripRow| with_origin_and_id(row))
    .map_err(convert_error)
}

pub async fn put<'c, E>(
    executor: E,
    line: WithOrigin<WithId<Trip>>,
) -> Result<WithOrigin<WithId<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO trips(
            id,
            origin,
            line_id,
            service_id,
            headsign,
            short_name
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id, origin)
        DO UPDATE SET
            line_id = EXCLUDED.line_id,
            service_id = EXCLUDED.service_id,
            headsign = EXCLUDED.headsign,
            short_name = EXCLUDED.short_name
        RETURNING *;
        ",
    )
    .bind(line.content.id.raw())
    .bind(line.origin.raw())
    .bind(line.content.content.line_id.raw())
    .bind(line.content.content.service_id.raw())
    .bind(line.content.content.headsign)
    .bind(line.content.content.short_name)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: TripRow| with_origin_and_id(row))
}

pub async fn update<'c, E>(
    _executor: E,
    _trip: WithOrigin<WithId<Trip>>,
) -> Result<WithOrigin<WithId<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    todo!()
}

pub async fn exists<'c, E>(executor: E, id: Id<Trip>) -> Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: use sql
    get(executor, id).await.map(|entry| entry.contains_data())
}

pub async fn exists_with_origin<'c, E>(
    executor: E,
    id: Id<Trip>,
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
) -> public_transport::database::Result<Option<Id<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::id_by_original_id(
        executor,
        origin,
        original_id,
        "trips_original_ids",
    )
    .await
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<Trip>,
) -> public_transport::database::Result<OriginalIdMapping<Trip>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::put_original_id(
        executor,
        origin,
        original_id,
        id,
        "trips_original_ids",
    )
    .await
}

// Trip Repo

pub async fn put_stop_time<'c, E>(
    executor: E,
    trip_id: Id<Trip>,
    stop_time: WithOrigin<StopTime>,
) -> Result<WithOrigin<StopTime>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO stop_times(
            origin,
            trip_id,
            stop_sequence,
            stop_id,
            arrival_time,
            departure_time,
            stop_headsign
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (origin, trip_id, stop_sequence)
        DO UPDATE SET
            stop_id = EXCLUDED.stop_id,
            arrival_time = EXCLUDED.arrival_time,
            departure_time = EXCLUDED.departure_time,
            stop_headsign = EXCLUDED.stop_headsign
        RETURNING *;
        ",
    )
    .bind(stop_time.origin.raw())
    .bind(trip_id.raw())
    .bind(stop_time.content.stop_sequence)
    .bind(stop_time.content.stop_id.raw())
    .bind(
        stop_time
            .content
            .arrival_time
            .map(|time| time.num_seconds()),
    )
    .bind(
        stop_time
            .content
            .departure_time
            .map(|time| time.num_seconds()),
    )
    .bind(stop_time.content.stop_headsign)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: StopTimeRow| {
        WithOrigin::new(Id::new(row.origin.clone()), row.to_model())
    })
}

pub async fn get_stop_times<'c, E>(
    executor: E,
    trip_id: Id<Trip>,
    origin: Id<Origin>,
) -> Result<Vec<StopTime>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            origin, trip_id, stop_sequence, stop_id, arrival_time, departure_time, stop_headsign
        FROM
            stop_times
        WHERE
            trip_id = $1 AND origin = $2;
        ",
    )
    .bind(trip_id.raw())
    .bind(origin.raw())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .into_iter()
    .map(|stop_time: StopTimeRow| stop_time.to_model())
    .collect::<Vec<_>>()
    .let_owned(|result| Ok(result))
}

pub async fn delete_stop_times<'c, E>(
    executor: E,
    trip_id: Id<Trip>,
    origin: Id<Origin>,
) -> Result<()>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query(
        "
        DELETE FROM
            stop_times
        WHERE
            trip_id = $1 AND origin = $2;
        ",
    )
    .bind(trip_id.raw())
    .bind(origin.raw())
    .execute(executor)
    .await
    .map_err(convert_error)?;
    Ok(())
}

pub async fn get_all_via_stop<'c, E>(
    executor: E,
    stops: &[&Id<Stop>],
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<Vec<DatabaseEntry<Trip>>>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: diese query optimieren!
    sqlx::query_as(
        "
        SELECT DISTINCT
            t.id, t.origin, t.line_id, t.service_id, t.headsign, t.short_name
        FROM
            trips t
            JOIN stop_times st ON t.id = st.trip_id
            JOIN stops s ON st.stop_id = s.id
            LEFT JOIN calendar_windows c ON t.service_id = c.service_id
        WHERE s.id = ANY($1)
          AND ((c.start_date <= $2::date AND c.end_date >= $3::date)
               OR EXISTS (
                   SELECT 1 FROM calendar_dates cd
                   WHERE cd.service_id = t.service_id
                     AND cd.date BETWEEN $2::date AND $3::date
                     AND cd.exception_type = 'added'));
        ",
    )
    .bind(stops.raw_ref::<str>())
    .bind(start.date_naive())
    .bind(end.date_naive())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|trips: Vec<TripRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(trips)))
    })
}

pub async fn merge_candidates<'c, E>(
    executor: E,
    trip: &Trip,
) -> Result<Vec<WithOrigin<WithId<Trip>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        WITH stop_trip_counts AS (
            SELECT
                st.trip_id,
                COUNT(DISTINCT st.stop_id) AS matching_stops
            FROM stop_times st
            WHERE st.stop_id = ANY($1)  -- $1 ist die Liste der stop_ids
            GROUP BY st.trip_id
        )
        SELECT
            t.*  -- Hier werden alle Spalten von trips zurückgegeben
        FROM
            stop_trip_counts stc
            JOIN
                trips t ON stc.trip_id = t.trip_id  -- Join mit der trips Tabelle
        WHERE
            stc.matching_stops = (SELECT COUNT(DISTINCT stop_id) FROM unnest($1) AS stop_id);
        ",
    )
    .bind(&trip.stops.iter().filter_map(|st| st.stop_id.clone().raw()).collect::<Vec<_>>())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|stops: Vec<TripRow>| Ok(with_origins_and_ids(stops)))
}
