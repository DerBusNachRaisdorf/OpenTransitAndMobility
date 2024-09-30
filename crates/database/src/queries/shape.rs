use model::{
    origin::Origin,
    shape::{Shape, ShapePoint},
    trip::{StopTime, Trip},
};
use public_transport::database::Result;
use utility::{id::Id, let_also::LetAlso};

use crate::data_model::{shape::ShapePointRow, trip::StopTimeRow};
use sqlx::{Executor, Postgres};

use super::convert_error;

pub async fn put_shape_point<'c, E>(
    executor: E,
    shape_id: Id<Shape>,
    shape_point_sequence: i32,
    shape_point: ShapePoint,
) -> Result<ShapePoint>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO shapes(
            id,
            sequence,
            latitude,
            longitude,
            distance
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id, sequence)
        DO UPDATE SET
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            distance = EXCLUDED.distance
        RETURNING *;
        ",
    )
    .bind(shape_id.raw())
    .bind(shape_point_sequence)
    .bind(shape_point.latitude)
    .bind(shape_point.longitude)
    .bind(shape_point.distance)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: ShapePointRow| row.to_model())
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
            id,
            sequence,

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
