use model::{
    origin::{Origin, OriginalIdMapping},
    stop::Stop,
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::Result;
use utility::{
    geo::{self, EARTH_RADIUS_KM},
    id::{Id, IdWrapper},
    let_also::LetAlso,
};

use crate::data_model::{
    stop::StopRow, with_origin_and_id, with_origins, with_origins_and_ids,
};
use sqlx::{Executor, Postgres};

use super::convert_error;

// Repo

pub async fn get<'c, E>(executor: E, id: Id<Stop>) -> Result<DatabaseEntry<Stop>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops
        WHERE id = $1;
        ",
    )
    .bind(&id.raw())
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|stops: Vec<StopRow>| {
        Ok(DatabaseEntry::gather(id, with_origins(stops)))
    })
}

pub async fn get_all<'c, E>(executor: E) -> Result<Vec<DatabaseEntry<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops;
        ",
    )
    .fetch_all(executor)
    .await
    .map_err(|why| convert_error(why))?
    .let_owned(|stops: Vec<StopRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(stops)))
    })
}

pub async fn insert<'c, E>(
    executor: E,
    stop: WithOrigin<Stop>,
) -> Result<WithOrigin<WithId<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO stops(
            origin,
            name,
            description,
            parent_id,
            latitude,
            longitude,
            address,
            platform_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *;
        ",
    )
    .bind(stop.origin.raw())
    .bind(&stop.content.name)
    .bind(&stop.content.description)
    .bind(stop.content.parent_id.clone().raw())
    .bind(stop.content.latitude())
    .bind(stop.content.longitude())
    .bind(stop.content.address())
    .bind(stop.content.platform_code)
    .fetch_one(executor)
    .await
    .map(|row: StopRow| with_origin_and_id(row))
    .map_err(convert_error)
}

pub async fn put<'c, E>(
    executor: E,
    stop: WithOrigin<WithId<Stop>>,
) -> Result<WithOrigin<WithId<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        INSERT INTO stops(
            id,
            origin,
            name,
            description,
            parent_id,
            latitude,
            longitude,
            address,
            platform_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8)
        ON CONFLICT (id, origin)
        DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            parent_id = EXCLUDED.parent_id,
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            address = EXCLUDED.address,
            platform_code = EXCLUDED.platform_code
        RETURNING *;
        ",
    )
    .bind(stop.content.id.raw())
    .bind(stop.origin.raw())
    .bind(&stop.content.content.name)
    .bind(&stop.content.content.description)
    .bind(stop.content.content.parent_id.clone().raw())
    .bind(stop.content.content.latitude())
    .bind(stop.content.content.longitude())
    .bind(stop.content.content.address())
    .bind(stop.content.content.platform_code)
    .fetch_one(executor)
    .await
    .map_err(|why| convert_error(why))
    .map(|row: StopRow| with_origin_and_id(row))
}

pub async fn insert_all<'c, E>(executor: E, stops: &[WithOrigin<Stop>]) -> Result<u64>
where
    E: Executor<'c, Database = Postgres>,
{
    let _ = super::insert_all(
        executor,
        &[
            "origin",
            "name",
            "description",
            "parent_id",
            "latitude",
            "longitude",
            "address",
            "platform_code",
        ],
        stops,
        |query, stop| {
            query.bind(stop.origin.raw())
            // todo
        },
        &[],
    )
    .await
    .map_err(convert_error)
    .map(|result| result.rows_affected());
    todo!()
}

pub async fn update<'c, E>(
    executor: E,
    stop: WithOrigin<WithId<Stop>>,
) -> Result<WithOrigin<WithId<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "
        UPDATE stops
        SET name = $1,
            description = $2,
            parent_id = $3,
            latitude = $4,
            longitude = $5,
            address = $6,
            platform_code = $7
        WHERE origin = $8 AND id = $9
        RETURNING *;
        ",
    )
    .bind(&stop.content.content.name)
    .bind(&stop.content.content.description)
    .bind(stop.content.content.parent_id.clone().raw())
    .bind(stop.content.content.latitude())
    .bind(stop.content.content.longitude())
    .bind(stop.content.content.address())
    .bind(stop.content.content.platform_code)
    .bind(stop.origin.raw())
    .bind(stop.content.id.raw())
    .fetch_one(executor)
    .await
    .map_err(convert_error)
    .map(|row: StopRow| with_origin_and_id(row))
}

pub async fn exists<'c, E>(executor: E, id: Id<Stop>) -> Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // TODO: use sql
    get(executor, id).await.map(|entry| entry.contains_data())
}

pub async fn exists_with_origin<'c, E>(
    executor: E,
    id: Id<Stop>,
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
            .any(|stop| stop.origin == origin)
    })
}

// Subject Repo

pub async fn id_by_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
) -> public_transport::database::Result<Option<Id<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::id_by_original_id(
        executor,
        origin,
        original_id,
        "stops_original_ids",
    )
    .await
}

pub async fn put_original_id<'c, E>(
    executor: E,
    origin: Id<Origin>,
    original_id: String,
    id: Id<Stop>,
) -> public_transport::database::Result<OriginalIdMapping<Stop>>
where
    E: Executor<'c, Database = Postgres>,
{
    super::origin::put_original_id(
        executor,
        origin,
        original_id,
        id,
        "stops_original_ids",
    )
    .await
}

// Stop Repo

pub async fn get_nearby<'c, E>(
    executor: E,
    center_latitude: f64,
    center_longitude: f64,
    radius_km: f64,
) -> Result<Vec<DatabaseEntry<Stop>>>
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
                stops
            WHERE
                latitude BETWEEN $4 AND $5
                AND longitude BETWEEN $6 AND $7
        )
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops
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
    .let_owned(|stops: Vec<StopRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(stops)))
    })
}

pub async fn get_by_name<'c, E, S>(
    executor: E,
    name: S,
) -> Result<Vec<DatabaseEntry<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
    S: Into<String> + Send,
{
    sqlx::query_as(
        "
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops
        WHERE name ILIKE $1;
        ",
    )
    .bind(name.into())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|stops: Vec<StopRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(stops)))
    })
}

pub async fn search<'c, E, S>(
    executor: E,
    pattern: S,
) -> Result<Vec<DatabaseEntry<Stop>>>
where
    E: Executor<'c, Database = Postgres>,
    S: Into<String> + Send,
{
    let pattern: String = pattern.into().replace('%', "");
    let prefix_pattern = format!("{}%", pattern);
    let prefix_postfix_pattern = format!("%{}%", pattern);
    sqlx::query_as(
        "
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops
        WHERE
            name % $1 OR name ILIKE $3
        ORDER BY
            -- exact matches first
            CASE
                WHEN name = $1 THEN 1
                WHEN name ILIKE $2 THEN 2
                WHEN name ILIKE $3 THEN 3
                ELSE 4
            END ASC,
            -- then sort by similarity
            similarity(name, $1) DESC
        LIMIT 50; -- TODO: maybe insert a parameter for this.
        ",
    )
    .bind(pattern)
    .bind(prefix_pattern)
    .bind(prefix_postfix_pattern)
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|stops: Vec<StopRow>| {
        Ok(DatabaseEntry::gather_many(with_origins_and_ids(stops)))
    })
}

pub async fn merge_candidates<'c, E>(
    executor: E,
    stop: &Stop,
    excluded_origin: &Id<Origin>,
) -> Result<Vec<WithOrigin<WithId<Stop>>>>
where
    E: Executor<'c, Database = Postgres>,
{
    let (lat, lon, rad) = match &stop.location {
        Some(location) => (
            location.latitude,
            location.longitude,
            model::stop::DISTANCE_THRESHOLD_KM,
        ),
        _ => (0.0, 0.0, 0.0),
    };
    let ((min_lat, min_lon), (max_lat, max_lon)) =
        geo::calculate_bounding_box(lat, lon, rad);

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
                stops
            WHERE
                latitude BETWEEN $4 AND $5
                AND longitude BETWEEN $6 AND $7
        )
        SELECT
            id, origin, name, description, parent_id,
            latitude, longitude, address, platform_code
        FROM
            stops
        WHERE
            ((name != '' AND name % $9)
                OR (ABS($8 - 0.0) > 0.00001 AND id IN (
                    SELECT id FROM distance_calc WHERE distance < $8
                )))
            AND NOT EXISTS (
                SELECT 1 FROM stops s2
                WHERE s2.id = stops.id
                AND s2.origin = $10
            );
        ",
    )
    .bind(EARTH_RADIUS_KM)
    .bind(lat)
    .bind(lon)
    .bind(min_lat)
    .bind(max_lat)
    .bind(min_lon)
    .bind(max_lon)
    .bind(rad)
    .bind(stop.name.clone().unwrap_or("".to_owned()))
    .bind(excluded_origin.raw_ref::<str>())
    .fetch_all(executor)
    .await
    .map_err(convert_error)?
    .let_owned(|stops: Vec<StopRow>| Ok(with_origins_and_ids(stops)))
}
