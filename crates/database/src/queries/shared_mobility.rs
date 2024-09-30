use model::{
    origin::{Origin, OriginalIdMapping},
    shared_mobility::{SharedMobilityStation, Status},
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::Result;
use sqlx::{types::Json, Executor, Postgres};
use utility::{
    geo::{self, EARTH_RADIUS_KM},
    id::Id,
    let_also::LetAlso,
};

use crate::data_model::{
    shared_mobility::SharedMobilityStationRow, with_origins_and_ids, DatabaseRow as _,
};

use super::convert_error;

pub async fn get<'c, E>(
    executor: E,
    id: &Id<SharedMobilityStation>,
) -> Result<Vec<DatabaseEntry<SharedMobilityStation>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, name, latitude, longitude, capacity,
            rentail_uri_android, rentail_uri_ios, rental_uri_web,
            status
        FROM
            shared_mobility_stations
        WHERE
            trip_id = $1;
        ",
    )
    .bind(id.raw_ref::<str>())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|rows: Vec<SharedMobilityStationRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(rows)))
    })
}

pub async fn get_nearby<'c, E>(
    executor: E,
    center_latitude: f64,
    center_longitude: f64,
    radius_km: f64,
) -> Result<Vec<DatabaseEntry<SharedMobilityStation>>>
where
    E: Executor<'c, Database = Postgres>,
{
    let ((min_lat, min_lon), (max_lat, max_lon)) =
        geo::calculate_bounding_box(center_latitude, center_longitude, radius_km);

    sqlx::query_as(
        "
        WITH distance_calc AS (
            SELECT
                id,
                ($1 * ACOS(
                    COS(RADIANS($2)) * COS(RADIANS(latitude)) *
                    COS(RADIANS(longitude) - RADIANS($3)) +
                    SIN(RADIANS($2)) * SIN(RADIANS(latitude))
                )) AS distance
            FROM
                shared_mobility_stations
            WHERE
                latitude BETWEEN $4 AND $5
                AND longitude BETWEEN $6 AND $7
        )
        SELECT
            id, origin, name, latitude, longitude, capacity,
            rental_uri_android, rental_uri_ios, rental_uri_web,
            status
        FROM
            shared_mobility_stations
        WHERE
            id IN (
                SELECT id FROM distance_calc WHERE distance < $8
            );
        ",
    )
    .bind(EARTH_RADIUS_KM)
    .bind(center_latitude)
    .bind(center_longitude)
    .bind(min_lat)
    .bind(max_lat)
    .bind(min_lon)
    .bind(max_lon)
    .bind(radius_km)
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|stops: Vec<SharedMobilityStationRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(stops)))
    })
}

pub async fn update_status<'c, E>(
    executor: E,
    origin: &Id<Origin>,
    id: &Id<SharedMobilityStation>,
    status: Option<Status>,
) -> Result<()>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query(
        "
        UPDATE shared_mobility_stations
        SET status = $3
        WHERE id = $1 AND origin = $2
        RETURNING *;
        ",
    )
    .bind(id.raw_ref::<str>())
    .bind(origin.raw_ref::<str>())
    .bind(status.map(|s| Json(s)))
    .execute(executor)
    .await
    .map_err(convert_error)?;
    Ok(())
}

pub async fn put_all<'c, E>(
    executor: E,
    origin: &Id<Origin>,
    stations: &[WithId<SharedMobilityStation>],
) -> Result<WithOrigin<Vec<WithId<SharedMobilityStation>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::insert_all_returning(
        executor,
        "shared_mobility_stations",
        &[
            "id",
            "origin",
            "name",
            "latitude",
            "longitude",
            "capacity",
            "rental_uri_android",
            "rental_uri_ios",
            "rental_uri_web",
            "status",
        ],
        stations,
        |query, station| {
            query
                .bind(station.id.raw())
                .bind(origin.raw())
                .bind(station.content.name.clone())
                .bind(station.content.latitude)
                .bind(station.content.longitude)
                .bind(station.content.capacity as i32)
                .bind(station.content.rental_uris.android.clone())
                .bind(station.content.rental_uris.ios.clone())
                .bind(station.content.rental_uris.web.clone())
                .bind(station.content.status.clone().map(|s| Json(s)))
        },
        &["id", "origin"],
    )
    .await
    .map(|results: Vec<SharedMobilityStationRow>| {
        WithOrigin::new(
            origin.clone(),
            results
                .into_iter()
                .map(|station| {
                    WithId::new(Id::new(station.id.clone()), station.to_model())
                })
                .collect(),
        )
    })
    .map_err(convert_error)
}

// Subject Repo

pub async fn id_by_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
) -> public_transport::database::Result<Option<Id<SharedMobilityStation>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::id_by_original_id(
        executor,
        origin,
        original_id,
        "shared_mobility_stations_original_ids",
    )
    .await
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<SharedMobilityStation>,
) -> public_transport::database::Result<OriginalIdMapping<SharedMobilityStation>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::put_original_id(
        executor,
        origin,
        original_id,
        id,
        "shared_mobility_stations_original_ids",
    )
    .await
}
