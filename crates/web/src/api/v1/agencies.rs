use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, State},
    http::Method,
    routing::{get, on},
    Extension, Router,
};
use model::{agency::Agency, WithId};
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
        crate::api::v1::resource!("/agencies{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/schema", get(schema::<Agency>))
        .route("/:id", get(get_agency))
        .route("/", get(get_agencies))
        .layer(axum::middleware::from_fn(base_url_middleware))
        .with_state(state)
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

async fn get_agencies(
    OriginalUri(original_uri): OriginalUri,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<VecResponse<hateoas::Response<Agency>>> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .get_agencies(origins)
        .await
        .map(|agencies| {
            agencies
                .into_iter()
                .map(|agency| agency_hateoas(agency, base_url.clone()))
                .collect::<Vec<_>>()
                .let_owned(|data| VecResponse::non_paginated(data).hateoas().json())
        })
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

async fn get_agency(
    OriginalUri(original_uri): OriginalUri,
    Path(id): Path<String>,
    State(WebState { transit_client, .. }): State<WebState>,
    Extension(base_url): Extension<Arc<BaseUrl>>,
) -> HateoasResult<Agency> {
    let origins = transit_client.get_origin_ids().await?;
    transit_client
        .get_agency(Id::new(id), origins)
        .await
        .map(|agency| agency_hateoas(agency, base_url).json())
        .map_err(|why| {
            RouteErrorResponse::from(why)
                .with_method(&Method::GET)
                .with_uri(original_uri.path())
        })
}

pub(crate) fn agency_hateoas(
    agency: WithId<Agency>,
    base_url: Arc<BaseUrl>,
) -> hateoas::Response<Agency> {
    hateoas::Response::builder(agency.content, base_url)
        .link("self", resource!("/{}", agency.id.raw()))
        .build()
}
