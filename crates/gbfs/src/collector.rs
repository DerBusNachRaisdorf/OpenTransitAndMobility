use std::{error::Error, time::Duration};

use async_trait::async_trait;
use public_transport::{
    client::Client,
    collector::{Collector, Continuation},
    database::Database,
};
use serde::{Deserialize, Serialize};

pub struct StationsCollector {
    url: String,
}

impl StationsCollector {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self { url: url.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationsState {
    pub url: String,
}

#[async_trait]
impl Collector for StationsCollector {
    type Error = Box<dyn Error + Send + Sync>;
    type State = StationsState;

    fn unique_id() -> &'static str {
        "GBFS Stations"
    }

    fn from_state(state: Self::State) -> Self {
        Self { url: state.url }
    }

    async fn run<D: Database>(
        &mut self,
        client: &Client<D>,
        state: Self::State,
    ) -> Result<(Continuation, Self::State), Self::Error> {
        crate::insert_station_information(client.clone(), &self.url)
            .await
            .unwrap();
        Ok((Continuation::Exit, state))
    }

    fn tick(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 60 * 24 * 30))
    }
}

pub struct StatusCollector {
    url: String,
}

impl StatusCollector {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self { url: url.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusState {
    pub url: String,
}

#[async_trait]
impl Collector for StatusCollector {
    type Error = Box<dyn Error + Send + Sync>;
    type State = StatusState;

    fn unique_id() -> &'static str {
        "GBFS Status"
    }

    fn from_state(state: Self::State) -> Self {
        Self { url: state.url }
    }

    async fn run<D: Database>(
        &mut self,
        client: &Client<D>,
        state: Self::State,
    ) -> Result<(Continuation, Self::State), Self::Error> {
        crate::update_station_status(client.clone(), &self.url)
            .await
            .unwrap();
        Ok((Continuation::Continue, state))
    }

    fn tick(&self) -> Option<Duration> {
        Some(Duration::from_secs(60))
    }
}
