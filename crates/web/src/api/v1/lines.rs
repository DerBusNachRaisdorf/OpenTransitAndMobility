use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, Query, State},
    http::Method,
    routing::{get, on},
    Extension, Router,
};
use model::{line::Line, WithId};
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
        crate::api::v1::resource!("/lines{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/schema", get(schema::<Line>))
        .route("/:id", get(get_line))
        .route("/", get(get_lines))
        .layer(axum::middleware::from_fn(base_url_middleware))
        .with_state(state)
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

#[derive(Deserialize)]
struct LinesQuery {
    stop: Option<String>,
}

async fn get_lines(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<LinesQuery>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<Line>>> {
    let origins = transit_client.get_origin_ids().await?;
    // get at stop if query stops
    if let Some(stop) = params.stop {
        transit_client
            .get_lines_at_stop(&Id::new(stop), &origins)
            .await
    // otherwise get all
    } else {
        transit_client.get_lines(origins).await
    }
    .map(|lines| {
        lines
            .into_iter()
            .map(|line| line_hateoas(line, base_url.clone()))
            .collect::<Vec<_>>()
            .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
    })
    .map_err(|why| {
        RouteErrorResponse::from(why)
            .with_method(&Method::GET)
            .with_uri(original_uri.path())
    })
}

async fn get_line(
    OriginalUri(original_uri): OriginalUri,
    Path(id): Path<String>,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<Line> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .get_line(Id::new(id), origins)
        .await
        .map(|line| line_hateoas(line, base_url).json())
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

pub(crate) fn line_hateoas(
    line: WithId<Line>,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<Line> {
    let agency_id = line.content.agency_id.clone();
    hateoas::Response::builder(line.content, base_url)
        .link("self", resource!("/{}", line.id.raw()))
        .link_option(
            "agency",
            agency_id.map(|id| super::agencies::resource!("/{}", id)),
        )
        .build()
}
