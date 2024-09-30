use chrono::{DateTime, Duration, Local, NaiveDate};
use model::{
    origin::Origin,
    trip::Trip,
    trip_update::{TripUpdate, TripUpdateId},
    DatabaseEntry, DateTimeRange, WithId, WithOrigin,
};
use public_transport::database::Result;
use sqlx::{types::Json, Executor, Postgres};
use utility::{
    id::{Id, IdWrapper as _},
    let_also::LetAlso,
};

use crate::data_model::{
    trip_update::{TripStatus, TripUpdateRow},
    with_origins, with_origins_and_ids, DatabaseRow as _,
};

use super::convert_error;

pub async fn get<'c, E>(
    executor: E,
    trip_id: &Id<Trip>,
    trip_start_date: NaiveDate,
) -> Result<DatabaseEntry<TripUpdate>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            origin, trip_id, trip_start_date, status, stop_time_updates, timestamp
        FROM
            trip_updates
        WHERE
            trip_id = $1;
        ",
    )
    .bind(trip_id.raw_ref::<str>())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|updates: Vec<TripUpdateRow>| {
        Ok(DatabaseEntry::gather(
            Id::new(TripUpdateId::new(trip_id.clone(), trip_start_date)),
            with_origins(updates),
        ))
    })
}

pub async fn get_for_trips_in_range<'c, E>(
    executor: E,
    trip_ids: &[Id<Trip>],
    range: DateTimeRange<Local>,
) -> Result<Vec<DatabaseEntry<TripUpdate>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            origin, trip_id, trip_start_date, status, stop_time_updates, timestamp
        FROM
            trip_updates
        WHERE
            trip_id = ANY($1)
            AND (trip_start_date BETWEEN $2::date AND $3::date);
        ",
    )
    .bind(trip_ids.raw_ref::<str>())
    .bind(range.first - Duration::days(1))
    .bind(range.last)
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|updates: Vec<TripUpdateRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(updates)))
    })
}

pub async fn get_timestamp<'c, E>(
    executor: E,
    origin: &Id<Origin>,
    trip_id: &Id<Trip>,
    trip_start_date: NaiveDate,
) -> Result<Option<DateTime<Local>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "
        SELECT
            timestamp
        FROM
            trip_updates
        WHERE
            origin = $1
            AND trip_id = $2
            AND trip_start_date = $3
        ",
    )
    .bind(origin.raw_ref::<str>())
    .bind(trip_id.raw_ref::<str>())
    .bind(trip_start_date)
    .fetch_optional(executor)
    .await
    .map(|result: Option<Option<DateTime<Local>>>| result.and_then(|x| x))
    .map_err(|why| convert_error(why))
}

pub async fn put_all<'c, E>(
    executor: E,
    origin: &Id<Origin>,
    updates: &[WithId<TripUpdate>],
) -> Result<WithOrigin<Vec<WithId<TripUpdate>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::insert_all_returning(
        executor,
        "trip_updates",
        &[
            "origin",
            "trip_id",
            "trip_start_date",
            "status",
            "stop_time_updates",
            "timestamp",
        ],
        updates,
        |query, update| {
            query
                .bind(origin.raw())
                .bind(update.id.raw().trip_id.raw())
                .bind(update.id.raw().trip_start_date)
                .bind(TripStatus::from(update.content.status.clone()))
                .bind(Json(update.content.stops.clone()))
                .bind(update.content.timestamp.clone())
        },
        &["origin", "trip_id", "trip_start_date"],
    )
    .await
    .map(|results: Vec<TripUpdateRow>| {
        WithOrigin::new(
            origin.clone(),
            results
                .into_iter()
                .map(|update| {
                    WithId::new(
                        Id::new(TripUpdateId::new(
                            Id::new(update.trip_id.clone()),
                            update.trip_start_date,
                        )),
                        update.to_model(),
                    )
                })
                .collect(),
        )
    })
    .map_err(convert_error)
}
