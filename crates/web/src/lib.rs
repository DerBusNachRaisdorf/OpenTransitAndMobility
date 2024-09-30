pub use crate::common::RouteResult;

use axum::{extract::FromRef, routing::get_service, Router};
use database::PgDatabase;
use public_transport::client::Client;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub mod api;
pub mod common;
pub mod hateoas;
pub mod middleware;

#[derive(Clone, FromRef)]
pub struct WebState {
    pub transit_client: Client<PgDatabase>,
}

pub async fn start_web_server(state: WebState) -> std::io::Result<()> {
    let routes = Router::new()
        .nest_service("/api", api::routes(state))
        .fallback_service(static_content_router());

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, routes.into_make_service()).await?;

    Ok(())
}

fn static_content_router() -> Router {
    Router::new().nest_service(
        "/",
        get_service(
            ServeDir::new("./resources/www/")
                .not_found_service(ServeFile::new("./resources/www/error404.html")),
        ),
    )
}
