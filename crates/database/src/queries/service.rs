use model::{
    calendar::{CalendarDate, CalendarWindow, Service},
    origin::{Origin, OriginalIdMapping},
};
use public_transport::database::Result;
use utility::{
    id::{Id, IdWrapper},
    let_also::LetAlso,
};

use crate::data_model::{
    calendar::{CalendarWindowRow, ServiceAvailability},
    calendar_exception::{CalendarDateRow, ServiceExceptionType},
    origin::OriginalIdMappingRow,
};
use sqlx::{Executor, Postgres};

use super::convert_error;

// Subject Repo

pub async fn id_by_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
) -> public_transport::database::Result<Option<Id<Service>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "
        SELECT
            id
        FROM
            services_original_ids
        WHERE
            origin = $1 AND original_id = $2;
        ",
    )
    .bind(origin.raw())
    .bind(original_id)
    .fetch_optional(executor)
    .await
    .map_err(convert_error)?
    .map(|id: i32| Id::new(id))
    .let_owned(|result| Ok(result))
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<Service>,
) -> public_transport::database::Result<OriginalIdMapping<Service>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO services_original_ids(
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
    )
    .bind(origin.raw())
    .bind(original_id)
    .bind(id.raw())
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: OriginalIdMappingRow<i32>| row.to_model())
}

// Service Repo

pub async fn get_calendar_windows<'c, E>(
    executor: E,
    id: &Id<Service>,
) -> Result<Vec<CalendarWindow>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            service_id,
            monday, tuesday, wednesday, thursday, friday, saturday, sunday,
            start_date, end_date
        FROM
            calendar_windows
        WHERE
            service_id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|windows: Vec<CalendarWindowRow>| {
        Ok(windows
            .into_iter()
            .map(|window| window.to_model())
            .collect::<Vec<_>>())
    })
}

pub async fn get_calendar_dates<'c, E>(
    executor: E,
    id: &Id<Service>,
) -> Result<Vec<CalendarDate>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            service_id, date, exception_type
        FROM
            calendar_dates
        WHERE
            service_id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|dates: Vec<CalendarDateRow>| {
        Ok(dates
            .into_iter()
            .map(|date| date.to_model())
            .collect::<Vec<_>>())
    })
}

pub async fn put_calendar_window<'c, E>(
    executor: E,
    service_id: Option<Id<Service>>,
    window: CalendarWindow,
) -> Result<(Id<Service>, CalendarWindow)>
where
    E: Executor<'c, Database = Postgres>,
{
    const INSERT_QUERY: &str = "
        INSERT INTO calendar_windows(
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            start_date,
            end_date
        )
        VALUES
            ($2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *;
        ";
    const PUT_QUERY: &str = "
        INSERT INTO calendar_windows(
            service_id,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            start_date,
            end_date
        )
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT
            (service_id, start_date, end_date)
        DO UPDATE SET
            monday = EXCLUDED.monday,
            tuesday = EXCLUDED.tuesday,
            wednesday = EXCLUDED.wednesday,
            thursday = EXCLUDED.thursday,
            friday = EXCLUDED.friday,
            saturday = EXCLUDED.saturday,
            sunday = EXCLUDED.sunday
        RETURNING *;
        ";
    sqlx::query_as(if service_id.is_none() {
        INSERT_QUERY
    } else {
        PUT_QUERY
    })
    .bind(service_id.raw())
    .bind(ServiceAvailability::from(window.monday))
    .bind(ServiceAvailability::from(window.tuesday))
    .bind(ServiceAvailability::from(window.wednesday))
    .bind(ServiceAvailability::from(window.thursday))
    .bind(ServiceAvailability::from(window.friday))
    .bind(ServiceAvailability::from(window.saturday))
    .bind(ServiceAvailability::from(window.sunday))
    .bind(&window.start_date)
    .bind(&window.end_date)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: CalendarWindowRow| {
        (
            Id::new(row.service_id.expect("service_id present in database.")),
            row.to_model(),
        )
    })
}

pub async fn put_calendar_date<'c, E>(
    executor: E,
    service_id: Option<Id<Service>>,
    date: CalendarDate,
) -> Result<(Id<Service>, CalendarDate)>
where
    E: Executor<'c, Database = Postgres>,
{
    const INSERT_QUERY: &str = "
        INSERT INTO calendar_dates(
            date,
            exception_type
        )
        VALUES
            ($2, $3)
        RETURNING *;
        ";
    const PUT_QUERY: &str = "
        INSERT INTO calendar_dates(
            service_id,
            date,
            exception_type
        )
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (service_id, date)
        DO UPDATE SET
            service_id = EXCLUDED.service_id,
            date = EXCLUDED.date,
            exception_type = EXCLUDED.exception_type
        RETURNING *;
        ";
    sqlx::query_as(if service_id.is_none() {
        INSERT_QUERY
    } else {
        PUT_QUERY
    })
    .bind(service_id.raw())
    .bind(date.date)
    .bind(ServiceExceptionType::from(date.exception_type))
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: CalendarDateRow| {
        (
            Id::new(row.service_id.expect("service_id present in database.")),
            row.to_model(),
        )
    })
}
