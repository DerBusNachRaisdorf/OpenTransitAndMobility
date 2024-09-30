use agencies::agency_hateoas;
use itertools::Itertools;
use lines::line_hateoas;
use schemars::JsonSchema;
use std::sync::Arc;
use stops::stop_with_distance_hateoas;

use crate::{
    common::{
        route_not_found, route_not_implemented, schema_no_example, HateoasResult,
        RouteErrorResponse, METHOD_FILTER_ALL,
    },
    hateoas,
    middleware::base_url::{base_url_middleware, BaseUrl},
    WebState,
};
use axum::{
    extract::{OriginalUri, Query, State},
    http::Method,
    routing::{get, on},
    Extension, Router,
};
use model::{
    line::Line, shared_mobility::SharedMobilityStation, stop::Stop,
    trip_instance::TripInstance, DateTimeRange, WithDistance,
};
use std::time::Instant;
use trips::{stop_time_hateoas, trip_hateoas, TripInstanceDto};
use utility::serde::date_time;

mod agencies;
mod lines;
mod realtime;
mod stops;
mod trips;

macro_rules! resource {
    ($($arg:tt)*) => {
        crate::api::resource!("/v1{}", format_args!($($arg)*))
    };
}
use chrono::{DateTime, Duration, Local};
pub(crate) use resource;
use serde::{Deserialize, Serialize};

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/", get(route_not_implemented))
        .route("/nearby", get(nearby))
        .route("/nearby/schema", get(schema_no_example::<NearbyDto>))
        .nest_service("/agencies", agencies::routes(state.clone()))
        .nest_service("/lines", lines::routes(state.clone()))
        .nest_service("/trips", trips::routes(state.clone()))
        .nest_service("/stops", stops::routes(state.clone()))
        .nest_service("/realtime", realtime::routes(state.clone()))
        .layer(axum::middleware::from_fn(base_url_middleware))
        .with_state(state)
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct NearbyDto {
    radius: f64,
    latitude: f64,
    longitude: f64,
    start: DateTime<Local>,
    end: DateTime<Local>,
    stops: Vec<hateoas::Response<WithDistance<Stop>>>,
    lines: Vec<hateoas::Response<Line>>,
    trips: Vec<hateoas::Response<TripInstanceDto>>,
    shared_mobility_stations: Vec<SharedMobilityStation>,
}

#[derive(Deserialize)]
pub(crate) struct TripsNearbyQuery {
    latitude: f64,

    longitude: f64,

    radius: Option<f64>,

    #[serde(deserialize_with = "date_time::deserialize_local_option", default)]
    start: Option<DateTime<Local>>,

    #[serde(deserialize_with = "date_time::deserialize_local_option", default)]
    end: Option<DateTime<Local>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NearbyBenchmark {
    fetch_shared_mobility_stations_secs: f64,
    fetch_stops_secs: f64,
    fetch_lines_secs: f64,
    fetch_trips_secs: f64,
    instantiate_trips_secs: f64,
    num_trips_fetched: usize,
}

async fn nearby(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<TripsNearbyQuery>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<NearbyDto> {
    let origins = transit_client.get_origin_ids().await?;
    let radius = params.radius.unwrap_or(0.05);
    let start = params.start.unwrap_or(Local::now());
    let end = params.end.unwrap_or(start + Duration::hours(1));

    // get shared mobility stations
    let now = Instant::now();
    let shared_mobility_stations = transit_client
        .find_nearby_shared_mobility_stations(
            params.latitude,
            params.longitude,
            radius,
            &origins,
        )
        .await
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_message("Could not query nearby shared mobility stations.")
                .with_uri(original_uri.path())
        })?;
    let fetch_shared_mobility_elapsed = now.elapsed();

    // get stops
    let now = Instant::now();
    let stops = transit_client
        .find_nearby(params.latitude, params.longitude, radius, &origins)
        .await
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_message("Could not query nearby stops.")
                .with_uri(original_uri.path())
        })?;
    let fetch_stops_elapsed = now.elapsed();

    // get lines and trips
    let now = Instant::now();
    let mut lines = vec![];
    for stop in stops.iter() {
        // get lines
        lines.extend(
            transit_client
                .get_lines_at_stop(&stop.content.id, &origins)
                .await
                .map_err(|why| {
                    RouteErrorResponse::from(why)
                        .with_method(&Method::GET)
                        .with_message("Could not query lines at nearby stops.")
                        .with_uri(original_uri.path())
                })?,
        );
    }
    let fetch_lines_elapsed = now.elapsed();

    // stop ids
    let stop_ids = stops
        .iter()
        .map(|stop| &stop.content.id)
        .collect::<Vec<_>>();

    // get raw trips
    // TODO: what to do with duplicate trips?
    let now = Instant::now();
    let trips = transit_client
        .get_all_trips_via_stops(&stop_ids, start, end, &origins)
        .await
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_message("Could not query trips at nearby stops.")
                .with_uri(original_uri.path())
        })?;
    let fetch_trips_elapsed = now.elapsed();
    let num_database_trips = trips.len();

    // instanciate trips
    let now = Instant::now();
    let mut instanciated_trips = transit_client
        .instanciate_trips_include(
            trips,
            DateTimeRange::new(start, end),
            Some(&stop_ids),
            true,
            true,
            true,
            &origins,
        )
        .await
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_message("Could not instanciate trips at nearby stops.")
                .with_uri(original_uri.path())
        })?;
    let instantiate_trips_elapsed = now.elapsed();

    // sort trips
    TripInstance::sort(&mut instanciated_trips);

    // unique lines
    lines = lines
        .into_iter()
        .unique_by(|line| line.id.clone())
        .collect();

    let benchmark = NearbyBenchmark {
        fetch_shared_mobility_stations_secs: fetch_shared_mobility_elapsed
            .as_secs_f64(),
        fetch_stops_secs: fetch_stops_elapsed.as_secs_f64(),
        fetch_lines_secs: fetch_lines_elapsed.as_secs_f64(),
        fetch_trips_secs: fetch_trips_elapsed.as_secs_f64(),
        instantiate_trips_secs: instantiate_trips_elapsed.as_secs_f64(),
        num_trips_fetched: num_database_trips,
    };

    let nearby = NearbyDto {
        radius,
        latitude: params.latitude,
        longitude: params.longitude,
        start,
        end,
        stops: stops
            .into_iter()
            .map(|stop| stop_with_distance_hateoas(stop, base_url.clone()))
            .collect(),
        lines: lines
            .into_iter()
            .map(|line| line_hateoas(line, base_url.clone()))
            .collect(),
        trips: instanciated_trips
            .into_iter()
            .map(|trip| {
                trip_hateoas(
                    TripInstanceDto {
                        info: trip.info,
                        stops: trip
                            .stops
                            .into_iter()
                            .map(|stop_time| {
                                stop_time_hateoas(stop_time, base_url.clone())
                            })
                            .collect::<Vec<_>>(),
                        stop_of_interest: trip.stop_of_interest,
                        line: trip
                            .line
                            .map(|line| line_hateoas(line, base_url.clone())),
                        agency: trip
                            .agency
                            .map(|agency| agency_hateoas(agency, base_url.clone())),
                    },
                    base_url.clone(),
                )
            })
            .collect::<Vec<_>>(),
        shared_mobility_stations: shared_mobility_stations
            .into_iter()
            .map(|x| x.content.content)
            .collect(),
    };

    Ok(nearby_hateoas(nearby, base_url, Some(benchmark)).json())
}

fn nearby_hateoas(
    dto: NearbyDto,
    base_url: Arc<BaseUrl>,
    benchmark: Option<NearbyBenchmark>,
) -> hateoas::Response<NearbyDto> {
    let latitude = dto.latitude;
    let longitude = dto.longitude;
    let radius = dto.radius;
    let start = dto.start.format("%Y-%m-%dT%H:%M:%S");
    let end = dto.end.format("%Y-%m-%dT%H:%M:%S");
    hateoas::Response::builder(dto, base_url)
        .link(
            "realtime",
            realtime::resource!(
                "/nearby?latitude={}&longitude={}&radius={}&start={}&end={}",
                latitude,
                longitude,
                radius,
                start,
                end
            ),
        )
        .debug_info_option("benchmark", benchmark)
        .build()
}
