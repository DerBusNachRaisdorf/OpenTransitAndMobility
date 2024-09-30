use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use model::{
    agency::Agency,
    calendar::CalendarDate,
    line::Line,
    stop::{Location, Stop},
    trip::{StopTime, Trip},
    trip_update::{StopTimeStatus, StopTimeUpdate},
};
use public_transport::{
    client::Client,
    collector::{Collector, Continuation},
    database::Database,
    RequestError,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{BahnApiClient, BahnApiCredentials},
    model::{
        station_data::SteamPermission,
        timetables::{EventStatus, TimetableStop},
    },
    station_data::get_station_data,
    timetables::{get_known_changes, get_plan},
};

/// The plan will be fetched in advance for this amount of hours (if alread provided).
/// TODO: maybe move into settings?
const MAX_PREFETCH_HOURS: i64 = 24 * 2;

fn is_ignored_trip_category(category: &str) -> bool {
    matches!(category, "erx" | "NBE" | "ME" | "AKN" | "Bus")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationState {
    pub eva: i64,
    pub last_plan_fetched: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorState {
    pub credentials: BahnApiCredentials,
    pub stations: Vec<StationState>,
}

pub struct DeutscheBahnCollector {
    client: Arc<BahnApiClient>,
    initialized: bool,
}

#[async_trait]
impl Collector for DeutscheBahnCollector {
    type Error = Box<dyn Error + Send + Sync>;
    type State = CollectorState;

    fn unique_id() -> &'static str {
        "DB Timetables"
    }

    fn from_state(state: Self::State) -> Self {
        Self {
            client: Arc::new(BahnApiClient::new(&state.credentials)),
            initialized: false,
        }
    }

    async fn run<D: Database>(
        &mut self,
        client: &Client<D>,
        mut state: Self::State,
    ) -> Result<(Continuation, Self::State), Self::Error> {
        // insert stations
        if !self.initialized {
            state = self.insert_stations(state, client).await?;
            self.initialized = true;
        }
        // insert planned trips
        state = self.insert_trips(client, state).await.unwrap();
        Ok((Continuation::Continue, state))
    }

    fn tick(&self) -> Option<Duration> {
        Some(Duration::from_secs(60))
    }
}

impl DeutscheBahnCollector {
    /// Inserts all stations into the database and returns the new station with
    /// all stations tracked.
    async fn insert_stations<D: Database>(
        &mut self,
        mut state: CollectorState,
        client: &Client<D>,
    ) -> Result<CollectorState, Box<dyn Error + Send + Sync>> {
        let mut station_states = state
            .stations
            .into_iter()
            .map(|state| (state.eva, state))
            .collect::<HashMap<_, _>>();
        // fetch stations from the DB StationData API
        let stada =
            get_station_data(self.client.clone(), "schleswig-holstein").await?;
        // insert the fetched stations into the database
        for station in stada.result {
            // get eva number
            let eva = match station.eva_numbers.first() {
                Some(number) => number,
                None => continue,
            };
            // steam permission emoji
            let dampf = if station.ril100_identifiers.iter().any(|ril100| {
                matches!(ril100.steam_permission, SteamPermission::Unrestricted)
            }) {
                "☁️"
            } else {
                ""
            };
            // build stop
            let stop = Stop {
                name: Some(station.name),
                description: station.product_line.map(|line| {
                    format!(
                        "{}{} ({}){}",
                        dampf, line.product_line, line.segment, dampf
                    )
                }),
                location: eva.geographic_coordinates.as_ref().map(|point| Location {
                    longitude: point.coordinates[0],
                    latitude: point.coordinates[1],
                    address: None,
                }),
                parent_id: None,
                platform_code: None,
            };
            // insert stop
            client
                .push_stop(stop, Some(format!("{}", eva.number)))
                .await
                .unwrap();
            // ensure stop in state
            if !station_states.contains_key(&eva.number) {
                station_states.insert(
                    eva.number,
                    StationState {
                        eva: eva.number,
                        last_plan_fetched: None,
                    },
                );
            }
        }
        state.stations = station_states.into_values().collect();
        Ok(state)
    }

    async fn insert_trips<D: Database>(
        &self,
        client: &Client<D>,
        mut state: CollectorState,
    ) -> Result<CollectorState, RequestError> {
        let mut front = vec![];
        let mut back = vec![];
        for mut station in state.stations {
            let now = Local::now();
            let next = station
                .last_plan_fetched
                .map(|last| {
                    let delta = now - last;
                    if delta > chrono::Duration::hours(4) {
                        now
                    } else {
                        last + chrono::Duration::hours(1)
                    }
                })
                .unwrap_or(now);
            let mut error = false;
            // fetch plan and insert
            if (next - now).num_hours() <= MAX_PREFETCH_HOURS {
                match get_plan(&self.client, station.eva, next).await {
                    Ok(timetable) => {
                        for mut stop in timetable.stops {
                            if stop.eva.is_none() {
                                stop.eva = Some(timetable.eva.unwrap_or(station.eva));
                            }
                            self.insert_planned_stop(client, stop).await?;
                        }
                        station.last_plan_fetched = Some(next);
                    }
                    Err(crate::ApiError::InvalidResponse { status_code, .. })
                        if matches!(status_code, StatusCode::NOT_FOUND) => {}
                    Err(why) => {
                        if !matches!(why, crate::ApiError::RateLimitReached) {
                            log::error!("{:?}", why);
                        }
                        error = true;
                    }
                }
            }
            // fetch updates
            match get_known_changes(&self.client, station.eva).await {
                Ok(timetable) => {
                    for stop in timetable.stops {
                        self.insert_stop_changes(client, stop).await?;
                    }
                }
                Err(why) => {
                    if !matches!(why, crate::ApiError::RateLimitReached) {
                        log::error!("{:?}", why);
                    }
                    error = true;
                }
            }

            if error {
                front.push(station);
            } else {
                back.push(station);
            }
        }
        state.stations = [front, back].concat();
        Ok(state)
    }

    async fn insert_planned_stop<D: Database>(
        &self,
        client: &Client<D>,
        stop: TimetableStop,
    ) -> Result<(), RequestError> {
        let Some(trip_label) = stop.trip_label else {
            return Ok(());
        };
        let Some(eva) = stop.eva else {
            return Ok(());
        };

        log::info!("inserting {}...", stop.id.full_id_string());

        let agency = client
            .push_agency(
                match trip_label.owner.as_str() {
                    "X1" => Agency {
                        name: "erixx".to_owned(),
                        website: "https://www.erixx.de/".to_owned(),
                        phone_number: None,
                        email: None,
                        fare_url: None,
                    },
                    "800292" => Agency {
                        name: "DB Regio AG Nord".to_owned(),
                        website: "https://www.bahn.de/".to_owned(),
                        phone_number: None,
                        email: None,
                        fare_url: None,
                    },
                    // TODO: there are a lot of EVUs missing.
                    other => Agency {
                        name: other.to_owned(),
                        website: "".to_owned(),
                        phone_number: None,
                        email: None,
                        fare_url: None,
                    },
                },
                Some(trip_label.owner.clone()),
            )
            .await?;

        let line_name = stop
            .arrival
            .as_ref()
            .and_then(|arrival| arrival.line.clone())
            .or(stop
                .departure
                .as_ref()
                .and_then(|departure| departure.line.clone()))
            .unwrap_or(trip_label.trip_or_train_number); // For ICE, IC, EC, etc.

        // ignore evu-specific trip categories like "erx" or "nbe"
        let line_name = if is_ignored_trip_category(trip_label.category.as_str()) {
            line_name
        } else {
            format!("{}{}", trip_label.category, line_name)
        };

        let kind = match trip_label.category.as_str() {
            "Bus" => model::line::LineType::Bus,
            _ => model::line::LineType::Rail,
        };

        let line = client
            .push_line(
                Line {
                    name: Some(line_name.clone()),
                    kind,
                    agency_id: Some(agency.content.id),
                },
                Some(format!("{}-{}", trip_label.owner, line_name)),
            )
            .await?;

        let date = stop
            .id
            .date()
            .map_err(|why| RequestError::Other(Box::new(why)))?;

        let service = client
            .push_calendar_date(
                None,
                CalendarDate {
                    date,
                    exception_type: model::calendar::ServiceExceptionType::Added,
                },
                Some(stop.id.trip_id_string()),
            )
            .await?;

        let trip = client
            .push_trip(
                Trip {
                    line_id: line.content.id,
                    service_id: Some(service.0),
                    headsign: None,
                    short_name: None,
                    stops: vec![],
                },
                Some(stop.id.trip_id_string()),
                false,
            )
            .await?;

        let stop_id = client
            .get_stop_id_by_original_id(format!("{}", eva))
            .await?;

        let Some(Some(date)) = service
            .1
            .date
            .and_hms_opt(0, 0, 0)
            .map(|x| x.and_local_timezone(Local).earliest())
        else {
            return Ok(());
        };
        client
            .push_stop_time(
                trip.content.id,
                StopTime {
                    stop_sequence: stop.id.index_of_stop_in_trip,
                    stop_id,
                    arrival_time: stop
                        .arrival
                        .as_ref()
                        .and_then(|arrival| arrival.planned_time)
                        .map(|pt| pt - date),
                    departure_time: stop
                        .departure
                        .as_ref()
                        .and_then(|departure| departure.planned_time)
                        .map(|pt| pt - date),
                    stop_headsign: None,
                },
            )
            .await?;

        log::info!(
            "inserted trip {} ({}): {}",
            stop.id.trip_id_string(),
            date,
            [
                stop.arrival.map(|a| a.planned_path).unwrap_or(vec![]),
                vec!["...".to_string()],
                stop.departure.map(|d| d.planned_path).unwrap_or(vec![]),
            ]
            .concat()
            .join(" | ")
        );

        Ok(())
    }

    async fn insert_stop_changes<D: Database>(
        &self,
        client: &Client<D>,
        stop: TimetableStop,
    ) -> Result<(), RequestError> {
        // TODO: when id does not exist, create new trip and insert anyway.
        // This would enable to also display added, unscheduled trips.
        let Some(id) = client
            .get_trip_id_by_original_id(stop.id.trip_id_string())
            .await?
        else {
            log::info!(
                "skipped update {}: {}",
                stop.id.trip_id_string(),
                serde_json::to_string_pretty(&stop).unwrap_or("hä".to_owned())
            );
            return Ok(());
        };

        let date = stop
            .id
            .date()
            .map_err(|why| RequestError::Other(Box::new(why)))?;

        client
            .put_stop_time_update(
                &id,
                date,
                StopTimeUpdate {
                    scheduled_stop_sequence: Some(stop.id.index_of_stop_in_trip),
                    arrival_time: stop.arrival.as_ref().and_then(|a| a.changed_time),
                    departure_time: stop
                        .departure
                        .as_ref()
                        .and_then(|d| d.changed_time),
                    status: stop
                        .arrival
                        .as_ref()
                        .and_then(|a| a.changed_status.clone())
                        .or(stop
                            .departure
                            .as_ref()
                            .and_then(|d| d.changed_status.clone()))
                        .map(|status| match status {
                            EventStatus::Added => StopTimeStatus::Added,
                            EventStatus::Planned => StopTimeStatus::Scheduled,
                            EventStatus::Cancelled => StopTimeStatus::Cancelled,
                        })
                        .unwrap_or(StopTimeStatus::Unknown),
                },
            )
            .await?;

        Ok(())
    }
}
