use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, Query, State},
    http::Method,
    routing::{get, on},
    Extension, Router,
};
use model::{
    stop::{Stop, StopNameSuggestion},
    WithDistance, WithId,
};
use serde::Deserialize;
use utility::{id::Id, let_also::LetAlso};

use crate::{
    common::{
        route_not_found, schema, HateoasResult, RouteErrorResponse, VecResponse,
        METHOD_FILTER_ALL,
    },
    hateoas,
    middleware::base_url::{base_url_middleware, BaseUrl},
    WebState,
};

macro_rules! resource {
    ($($arg:tt)*) => {
        crate::api::v1::resource!("/stops{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/schema", get(schema::<Stop>))
        .route("/:id", get(get_stop))
        .route("/", get(get_stops))
        .route("/search/:name", get(search_stop))
        .route("/nearby", get(nearby))
        .layer(axum::middleware::from_fn(base_url_middleware))
        .with_state(state)
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

async fn get_stops(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<Stop>>> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .get_stops(origins)
        .await
        .map(|stops| {
            stops
                .into_iter()
                .map(|stop| stop_hateoas(stop, base_url.clone()))
                .collect::<Vec<_>>()
                .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
        })
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

async fn get_stop(
    OriginalUri(original_uri): OriginalUri,
    Path(id): Path<String>,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<Stop> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .get_stop(Id::new(id), origins)
        .await
        .map(|stop| stop_hateoas(stop, base_url.clone()).json())
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

async fn search_stop(
    OriginalUri(original_uri): OriginalUri,
    Path(pattern): Path<String>,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<StopNameSuggestion>>> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .search_stop(pattern, &origins)
        .await
        .map(|stops| {
            stops
                .into_iter()
                .map(|stop| stop_suggestion_hateoas(stop, base_url.clone()))
                .collect::<Vec<_>>()
                .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
        })
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

#[derive(Deserialize)]
struct NearbyQuery {
    latitude: f64,
    longitude: f64,
    radius: Option<f64>,
}

async fn nearby(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<NearbyQuery>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<WithDistance<Stop>>>> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .find_nearby(
            params.latitude,
            params.longitude,
            params.radius.unwrap_or(0.05),
            &origins,
        )
        .await
        .map(|stops| {
            stops
                .into_iter()
                .map(|stop| stop_with_distance_hateoas(stop, base_url.clone()))
                .collect::<Vec<_>>()
                .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
        })
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

fn stop_hateoas(
    stop: WithId<Stop>,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<Stop> {
    let location = stop.content.location.clone();
    hateoas::Response::builder(stop.content, base_url)
        .link("self", resource!("/{}", stop.id.raw()))
        .link("trips", super::trips::resource!("?stop={}", stop.id.raw()))
        .link_option(
            "nearby",
            location.map(|location| {
                resource!(
                    "/nearby?latitude={}&longitude={}&radius=1",
                    location.latitude,
                    location.longitude
                )
            }),
        )
        .build()
}

pub fn stop_with_distance_hateoas(
    stop: WithDistance<WithId<Stop>>,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<WithDistance<Stop>> {
    let id = &stop.content.id;
    hateoas::Response::builder(
        WithDistance::new(stop.distance_km, stop.content.content),
        base_url,
    )
    .link("self", resource!("/{}", id.raw()))
    .link("trips", super::trips::resource!("?stop={}", id.raw()))
    .link("lines", super::lines::resource!("?stop={}", id.raw()))
    .build()
}

fn stop_suggestion_hateoas(
    stop: StopNameSuggestion,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<StopNameSuggestion> {
    let id = stop.id.clone();
    let latitude = stop.latitude.clone();
    let longitude = stop.longitude.clone();
    hateoas::Response::builder(stop, base_url)
        .link("self", resource!("/{}", id.raw()))
        .link("lines", super::lines::resource!("?stop={}", id.raw()))
        .link(
            "nearby",
            resource!(
                "/nearby?latitude={}&longitude={}&radius=1",
                latitude,
                longitude
            ),
        )
        .build()
}
