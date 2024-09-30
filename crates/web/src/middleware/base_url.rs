use axum::{
    extract::{self},
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BaseUrl {
    proto: String,
    host: String,
    prefix: String,
}

impl BaseUrl {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let proto = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let host = headers
            .get("x-forwarded-host")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("host").and_then(|v| v.to_str().ok()))
            .unwrap_or("localhost")
            .to_string();

        let prefix = headers
            .get("x-forwarded-prefix")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        BaseUrl {
            proto,
            host,
            prefix,
        }
    }

    pub fn full_url<S: Into<String>>(&self, path: S) -> String {
        format!(
            "{}://{}{}{}",
            self.proto,
            self.host,
            self.prefix,
            path.into()
        )
    }
}

pub async fn base_url_middleware(req: extract::Request, next: Next) -> impl IntoResponse {
    let headers = req.headers().clone();
    let base_url = BaseUrl::from_headers(&headers);

    let mut req = req;
    req.extensions_mut().insert(Arc::new(base_url));

    next.run(req).await
}
