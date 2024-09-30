use axum::{
    extract::{OriginalUri, Query, Request},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::MethodFilter,
    Json,
};
use model::ExampleData;
use public_transport::RequestError;
use schemars::{schema_for, schema_for_value, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::hateoas;

pub type RouteResult<O> = Result<O, RouteErrorResponse>;
pub type HateoasResult<O> = RouteResult<Json<hateoas::Response<O>>>;

/// A `MethodFilter` that matches all http methods.
pub(crate) const METHOD_FILTER_ALL: MethodFilter = MethodFilter::GET
    .or(MethodFilter::POST)
    .or(MethodFilter::PATCH)
    .or(MethodFilter::PUT)
    .or(MethodFilter::DELETE);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub page_size: usize,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VecResponse<T> {
    pub data: Vec<T>,
    pub pagination: Option<Pagination>,
}

impl<T> VecResponse<T> {
    pub fn non_paginated(data: Vec<T>) -> Self {
        Self {
            data,
            pagination: None,
        }
    }

    pub fn paginated(
        data: Vec<T>,
        current_page: usize,
        total_pages: usize,
        total_items: usize,
        page_size: usize,
    ) -> Self {
        Self {
            data,
            pagination: Some(Pagination {
                current_page,
                total_pages,
                total_items,
                page_size,
            }),
        }
    }

    pub fn hateoas(self) -> hateoas::Response<Self> {
        hateoas::Response::new(self)
    }

    pub fn json(self) -> Json<Self> {
        Json(self)
    }
}

// - Services returning commonly used responses -

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SchemaParams {
    #[serde(default = "Default::default")]
    example_data: bool,
}

pub(crate) async fn schema<T: ExampleData + JsonSchema + Serialize>(
    Query(params): Query<SchemaParams>,
) -> impl IntoResponse {
    if params.example_data {
        Json(schema_for_value!(T::example_data()))
    } else {
        Json(schema_for!(T))
    }
}

pub(crate) async fn schema_no_example<T: JsonSchema + Serialize>(
    Query(_params): Query<SchemaParams>,
) -> impl IntoResponse {
    Json(schema_for!(T))
}

pub(crate) async fn route_not_implemented(
    OriginalUri(original_uri): OriginalUri,
    req: Request,
) -> impl IntoResponse {
    not_implemented_response(req.method(), original_uri.path())
}

pub(crate) async fn route_not_found(
    OriginalUri(original_uri): OriginalUri,
    req: Request,
) -> impl IntoResponse {
    not_found_response(req.method(), original_uri.path())
}

// - Commonly used responeses -

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteErrorResponse {
    #[serde(skip)]
    pub status_code: StatusCode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_information: Option<String>,
}

impl RouteErrorResponse {
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            http_method: None,
            requested_uri: None,
            message: None,
            detailed_information: None,
        }
    }

    pub fn not_implemented(method: &Method, uri: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_IMPLEMENTED)
            .with_method(method)
            .with_uri(uri)
            .with_default_message()
    }

    pub fn not_found(method: &Method, uri: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND)
            .with_method(method)
            .with_uri(uri)
            .with_default_message()
    }

    pub fn with_method(mut self, method: &Method) -> Self {
        self.http_method = Some(method.to_string());
        self
    }

    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.requested_uri = Some(uri.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_default_message(self) -> Self {
        let message = self
            .status_code
            .canonical_reason()
            .unwrap_or("i dunno what happened here :/");
        self.with_message(message)
    }

    pub fn with_detailed_information(mut self, message: impl Into<String>) -> Self {
        self.detailed_information = Some(message.into());
        self
    }
}

impl From<RequestError> for RouteErrorResponse {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::NotFound => Self::new(StatusCode::BAD_REQUEST)
                .with_message("The requested item does not exist."),
            RequestError::Other(other) => {
                Self::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_message(format!("{}", other))
            }
            _ => Self::new(StatusCode::INTERNAL_SERVER_ERROR).with_default_message(),
        }
    }
}

impl IntoResponse for RouteErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, Json(self)).into_response()
    }
}

pub(crate) fn not_implemented_response(
    method: &Method,
    uri: &str,
) -> impl IntoResponse {
    RouteErrorResponse::not_implemented(method, uri)
}

pub(crate) fn not_found_response(method: &Method, uri: &str) -> impl IntoResponse {
    RouteErrorResponse::not_found(method, uri)
}
