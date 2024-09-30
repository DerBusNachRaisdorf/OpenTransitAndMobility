use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Query, State},
    http::{Method, StatusCode},
    routing::{get, on},
    Extension, Json, Router,
};
use chrono::{DateTime, Duration, Local};
use model::{
    agency::Agency,
    line::Line,
    trip::Trip,
    trip_instance::{StopTimeInstance, TripInstance, TripInstanceInfo},
    DateTimeRange, ExampleData, WithId,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utility::{id::Id, let_also::LetAlso, serde::date_time};

use crate::{
    common::{
        route_not_found, schema, HateoasResult, RouteErrorResponse, VecResponse,
        METHOD_FILTER_ALL,
    },
    hateoas,
    middleware::base_url::{base_url_middleware, BaseUrl},
    RouteResult, WebState,
};

macro_rules! resource {
    ($($arg:tt)*) => {
        crate::api::v1::resource!("/trips{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

use super::{agencies::agency_hateoas, lines::line_hateoas};

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/schema", get(schema::<TripInstanceDto>))
        .route("/", get(get_trips))
        .route("/debug", get(get_trips_debug))
        .layer(axum::middleware::from_fn(base_url_middleware))
        .with_state(state)
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

#[derive(Deserialize)]
struct TripsQuery {
    stop: Option<String>,

    #[serde(deserialize_with = "date_time::deserialize_local_option", default)]
    start: Option<DateTime<Local>>,

    #[serde(deserialize_with = "date_time::deserialize_local_option", default)]
    end: Option<DateTime<Local>>,
}

async fn get_trips_debug(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<TripsQuery>,
    Extension(_base_url): Extension<Arc<BaseUrl>>,
) -> RouteResult<Json<VecResponse<WithId<Trip>>>> {
    let origins = transit_client.get_origin_ids().await?;
    let start = params.start.unwrap_or(Local::now());
    let end = params.end.unwrap_or(start + Duration::hours(4));
    // get at stop if query stops
    if let Some(stop) = params.stop {
        let id = Id::new(stop);
        transit_client
            .get_all_trips_via_stops(&[&id], start, end, &origins)
            .await
            .map_err(|why| {
                RouteErrorResponse::from(why)
                    .with_method(&Method::GET)
                    .with_uri(original_uri.path())
            })?
            .let_owned(|trips| Ok(VecResponse::non_paginated(trips).json()))
    // otherwise get all
    } else {
        //transit_client.get_trips(origins).await
        return Err(RouteErrorResponse::new(StatusCode::BAD_REQUEST)
            .with_message("please narrow your request.")
            .with_method(&Method::GET)
            .with_uri(original_uri.path()));
    }
}

async fn get_trips(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<TripsQuery>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<TripInstanceDto>>> {
    let origins = transit_client.get_origin_ids().await?;
    let start = params.start.unwrap_or(Local::now());
    let end = params.end.unwrap_or(start + Duration::hours(4));
    // get at stop if query stops
    if let Some(stop) = params.stop {
        let id = Id::new(stop);
        let trips = transit_client
            .get_all_trips_via_stops(&[&id], start, end, &origins)
            .await
            .map_err(|why| {
                RouteErrorResponse::from(why)
                    .with_method(&Method::GET)
                    .with_uri(original_uri.path())
            })?;
        transit_client
            .instanciate_trips_include(
                trips,
                DateTimeRange::new(start, end),
                Some(&[&id]),
                true,
                true,
                true,
                &origins,
            )
            .await
    // otherwise get all
    } else {
        //transit_client.get_trips(origins).await
        return Err(RouteErrorResponse::new(StatusCode::BAD_REQUEST)
            .with_message("please narrow your request.")
            .with_method(&Method::GET)
            .with_uri(original_uri.path()));
    }
    .map(|trip_instances| {
        trip_instances
            .let_owned(TripInstance::sorted)
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
            .collect::<Vec<_>>()
            .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
    })
    .map_err(|why| {
        RouteErrorResponse::from(why)
            .with_method(&Method::GET)
            .with_uri(original_uri.path())
    })
}

pub fn trip_hateoas(
    trip: TripInstanceDto,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<TripInstanceDto> {
    let id = trip.info.trip_id.clone();
    hateoas::Response::builder(trip, base_url)
        .link("self", resource!("/{}", id.raw()))
        .build()
}

pub fn stop_time_hateoas(
    stop_time: StopTimeInstance,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<StopTimeInstance> {
    let id = stop_time.stop_id.clone();
    hateoas::Response::builder(stop_time, base_url)
        .link_option(
            "stop",
            id.map(|id| super::stops::resource!("/{}", id.raw())),
        )
        .build()
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TripInstanceDto {
    #[serde(flatten)]
    pub info: TripInstanceInfo,
    pub stops: Vec<hateoas::Response<StopTimeInstance>>,
    pub stop_of_interest: Option<StopTimeInstance>,
    pub line: Option<hateoas::Response<Line>>,
    pub agency: Option<hateoas::Response<Agency>>,
}

impl ExampleData for TripInstanceDto {
    fn example_data() -> Self {
        TripInstanceDto {
            info: TripInstanceInfo {
                trip_id: Id::new("eine-id".to_owned()),
                line_id: Id::new("eine-line".to_owned()),
                service_id: Some(Id::new(123)),
                headsign: Some("Moin Moin!".to_owned()),
                short_name: None,
            },
            stops: vec![], // TODO!
            stop_of_interest: None,
            line: None,
            agency: None,
        }
    }
}
