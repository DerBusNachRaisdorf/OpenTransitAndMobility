use std::env;

use serde::Deserialize;
use serde::Serialize;

use tokio::sync::RwLock;

use chrono::Local;

use crate::ApiError;

pub const BAHN_API_URL: &str =
    "https://apis.deutschebahn.com/db-api-marketplace/apis";

pub enum Accept {
    Xml,
    Json,
}

impl Accept {
    pub fn text(&self) -> String {
        match self {
            Self::Xml => "application/xml".to_owned(),
            Self::Json => "application/json".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BahnApiCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub rate_limit_per_minute: Option<u64>,
    pub proxy: Option<String>,
}

impl BahnApiCredentials {
    pub fn env() -> Self {
        let client_id = env::var("BAHN_CLIENT_ID").expect("Expected Bahn-Client-ID.");
        let client_secret =
            env::var("BAHN_CLIENT_SECRET").expect("Expected Bahn-Client-Secret.");

        Self {
            client_id,
            client_secret,
            rate_limit_per_minute: None,
            proxy: None,
        }
    }
}

#[derive(Clone, Default)]
struct BahnApiClientStats {
    pub sum_available_requests: u64,
    pub num_of_measurements: u64,
}

impl BahnApiClientStats {
    pub fn measure(&mut self, available_requests: u64) {
        self.sum_available_requests += available_requests;
        self.num_of_measurements += 1;
    }

    pub fn reset(&mut self) -> Self {
        let res = self.clone();
        self.sum_available_requests = 0;
        self.num_of_measurements = 0;
        res
    }
}

struct BahnApiClientState {
    pub avaliable_requests: u64,
    pub last_refill: chrono::DateTime<Local>,
}

pub struct BahnApiClient {
    pub credentials: BahnApiCredentials,
    state: RwLock<BahnApiClientState>,
    stats: RwLock<BahnApiClientStats>,
}

impl BahnApiClient {
    pub fn new(credentials: &BahnApiCredentials) -> Self {
        Self {
            credentials: credentials.clone(),
            state: RwLock::new(BahnApiClientState {
                avaliable_requests: credentials.rate_limit_per_minute.unwrap_or(0),
                last_refill: chrono::offset::Local::now(),
            }),
            stats: RwLock::new(BahnApiClientStats::default()),
        }
    }

    pub async fn stats_measure(&self) {
        let available_requests = self.avaliable_requests().await;
        self.stats.write().await.measure(available_requests);
    }

    pub async fn stats_reset(&self) -> u64 {
        let stats = self.stats.write().await.reset();
        stats.sum_available_requests / stats.num_of_measurements
    }

    pub async fn avaliable_requests(&self) -> u64 {
        self.state.read().await.avaliable_requests
    }

    async fn try_decrement_avaliable_requests(&self) -> Result<(), ApiError> {
        if let Some(rate_limit_minutes) = self.credentials.rate_limit_per_minute {
            let mut state = self.state.write().await;

            let minutes_since_last_request =
                (chrono::offset::Local::now() - state.last_refill).num_minutes();
            if minutes_since_last_request >= 1 {
                state.avaliable_requests = rate_limit_minutes;
                state.last_refill = chrono::offset::Local::now();
            }

            if state.avaliable_requests != 0 {
                state.avaliable_requests -= 1;
            } else {
                return Err(ApiError::RateLimitReached);
            }
        }
        Ok(())
    }

    /// Fetch data from an endpoint using this client.
    pub async fn get<'a, T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        accept: Accept,
    ) -> Result<T, ApiError> {
        self.try_decrement_avaliable_requests().await?;

        /* build a new http client with optional proxy */
        let client = if let Some(proxy_url) = &self.credentials.proxy {
            println!("Requesting Endpoint '{endpoint}' using proxy '{proxy_url}'.");
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(proxy_url)?)
                .build()?
        } else {
            println!("Requesting Endpoint '{endpoint}'.");
            reqwest::Client::new()
        };

        /* perform get-request */
        let url = format!("{BAHN_API_URL}/{endpoint}");
        let response = client
            .get(&url)
            .header("DB-Client-Id", &self.credentials.client_id)
            .header("DB-Api-Key", &self.credentials.client_secret)
            .header("accept", accept.text())
            .send()
            .await?;

        /* parse response */
        match response.status() {
            reqwest::StatusCode::OK => match accept {
                Accept::Xml => Ok(serde_xml_rs::from_str(&response.text().await?)?),
                Accept::Json => Ok(response.json().await?),
            },
            other => match response.text().await {
                Ok(val) => Err(ApiError::InvalidResponse {
                    status_code: other,
                    url,
                    response: Some(val),
                }),
                Err(_) => Err(ApiError::InvalidResponse {
                    status_code: other,
                    url,
                    response: None,
                }),
            },
        }
    }
}
