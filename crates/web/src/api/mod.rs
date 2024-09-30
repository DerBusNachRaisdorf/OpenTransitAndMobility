use axum::{
    response::IntoResponse,
    routing::{get, on},
    Json, Router,
};
use serde_json::json;

pub mod v1;

use crate::{
    common::{route_not_found, METHOD_FILTER_ALL},
    WebState,
};

macro_rules! resource {
    ($($arg:tt)*) => {
        format!("/api{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

pub fn routes(state: WebState) -> Router {
    Router::new()
        .route("/ping", get(ping))
        .nest_service("/v1", v1::routes(state))
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

async fn ping() -> impl IntoResponse {
    Json(json!({
        "message": "pong!"
    }))
}
