use phf::phf_map;
use std::error;
use std::fmt;
use std::sync::Arc;

pub mod client;
pub mod collector;
pub mod model;
pub mod station_data;
pub mod timetables;
pub mod triptable;

pub fn make_valid_station_name_key(station_name: &str) -> String {
    station_name
        .to_lowercase()
        .replace('ö', "oe")
        .replace('ä', "ae")
        .replace('ü', "ue")
        .replace('ß', "ss")
        .replace(['(', ')'], "")
        .replace(')', "")
        .replace(' ', "-")
}

/// table containing translations for problematic stations (with Umlaute)
pub static STATION_TABLE: phf::Map<&'static str, &'static str> = phf_map! {
    "plön" => "APLN",
    "ploen" => "APLN",
    "plon" => "APLN",
};

#[derive(Debug, Clone)]
pub enum ApiError {
    RequestError(Arc<reqwest::Error>),
    ParseError(Arc<serde_xml_rs::Error>),
    JsonError(Arc<serde_json::Error>),
    InvalidResponse {
        status_code: reqwest::StatusCode,
        url: String,
        response: Option<String>,
    },
    RateLimitReached,
    StationDoesNotExist(String),
    Other(String),
}

impl error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "HTTP request error: {}", e),
            ApiError::ParseError(e) => write!(f, "XML parse error: {}", e),
            ApiError::JsonError(e) => write!(f, "JSON parse error: {}", e),
            ApiError::InvalidResponse {
                status_code,
                url,
                response,
            } => match response {
                Some(text) => {
                    write!(f, "Invalid Response ({}) {}: {}", status_code, text, url)
                }
                None => write!(f, "Invalid Response({}) {}", status_code, url),
            },
            ApiError::StationDoesNotExist(s) => {
                write!(f, "Station does not exist: {}", s)
            }
            ApiError::RateLimitReached => write!(f, "Rate limit reached."),
            ApiError::Other(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::RequestError(Arc::new(e))
    }
}

impl From<serde_xml_rs::Error> for ApiError {
    fn from(e: serde_xml_rs::Error) -> Self {
        ApiError::ParseError(Arc::new(e))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::JsonError(Arc::new(e))
    }
}
