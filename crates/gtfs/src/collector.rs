use std::{error::Error, fs::File, path::Path, time::Duration};

use async_trait::async_trait;
use model::line::LineType;
use public_transport::{
    client::Client,
    collector::{Collector, Continuation},
    database::Database,
    RequestError,
};
use serde::{Deserialize, Serialize};
use utility::id::IdWrapper as _;

use crate::{
    data_model::{
        agency::Agency,
        calendar::CalendarRow,
        calendar_dates::CalendarDate,
        routes::{Route, RouteType},
        stop_times::StopTime,
        stops::Stop,
        trips::Trip,
    },
    download_gtfs,
    realtime::update,
};

pub struct RealtimeCollector {
    update: Duration,
}

impl RealtimeCollector {
    pub fn new<S: Into<String>>(update: Duration) -> Self {
        Self { update }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeCollectorState {
    pub url: String,
    pub update_interval: Duration,
}

#[async_trait]
impl Collector for RealtimeCollector {
    type Error = Box<dyn Error + Send + Sync>;
    type State = RealtimeCollectorState;

    fn unique_id() -> &'static str {
        "GTFS Realtime"
    }

    fn from_state(state: Self::State) -> Self {
        Self {
            update: state.update_interval,
        }
    }

    async fn run<D>(
        &mut self,
        client: &Client<D>,
        state: RealtimeCollectorState,
    ) -> Result<(Continuation, Self::State), Self::Error>
    where
        D: Database,
    {
        log::info!("update!");
        update(client.clone(), &state.url).await.unwrap();
        Ok((Continuation::Continue, state))
    }

    fn tick(&self) -> Option<Duration> {
        Some(self.update)
    }
}

pub struct ScheduleCollector {}

impl ScheduleCollector {
    pub fn new<S: Into<String>>() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleCollectorState {
    pub url: String,
}

#[async_trait]
impl Collector for ScheduleCollector {
    type Error = Box<dyn Error + Send + Sync>;
    type State = ScheduleCollectorState;

    fn unique_id() -> &'static str {
        "GTFS Schedule"
    }

    fn from_state(_state: Self::State) -> Self {
        Self {}
    }

    async fn run<D: Database>(
        &mut self,
        client: &Client<D>,
        state: Self::State,
    ) -> Result<(Continuation, Self::State), Self::Error> {
        download_and_insert(client, "", &state.url).await?;
        Ok((Continuation::Exit, state))
    }

    fn tick(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 60 * 24 * 30))
    }
}

async fn download_and_insert<D: Database, P: Into<String>, S: Into<String>>(
    client: &Client<D>,
    path_prefix: P,
    url: S,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("downloading gtfs...");
    download_gtfs(&url.into()).await?;
    println!("inserting gtfs tables...");
    insert_tables(client, Path::new("./").join(&path_prefix.into()).as_path())
        .await?
        .print();
    println!("gtfs complete.");
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct GtfsReport {
    skipped_agencies: usize,
    skipped_routes: usize,
    skipped_stops: usize,
    skipped_calendar_rows: usize,
    skipped_calendar_dates: usize,
    skipped_trips: usize,
    skipped_stop_times: usize,
}

impl GtfsReport {
    fn print(&self) {
        println!(
            "gtfs report: {}",
            serde_json::to_string_pretty(self).unwrap()
        );
    }
}

struct Progress {
    counter: usize,
    step: usize,
}

impl Progress {
    fn new(step: usize) -> Self {
        Self { counter: 0, step }
    }

    fn inc(&mut self) {
        self.counter += 1;
        if self.counter % self.step == 0 {
            log::info!("progress: {}", self.counter);
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}

async fn insert_tables<D: Database>(
    client: &Client<D>,
    path: &Path,
) -> Result<GtfsReport, Box<dyn Error + Send + Sync>> {
    let mut report = GtfsReport {
        skipped_agencies: 0,
        skipped_routes: 0,
        skipped_stops: 0,
        skipped_calendar_rows: 0,
        skipped_calendar_dates: 0,
        skipped_trips: 0,
        skipped_stop_times: 0,
    };
    let mut progress = Progress::new(1000);

    // agencies
    log::info!("inserting agencies...");
    let mut reader = csv::Reader::from_reader(File::open(path.join("agency.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_agency(client, row).await {
            report.skipped_agencies += 1;
        }
        progress.inc();
    }
    progress.reset();

    // routes
    log::info!("inserting routes...");
    let mut reader = csv::Reader::from_reader(File::open(path.join("routes.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_route(client, row).await {
            report.skipped_routes += 1;
        }
        progress.inc();
    }

    // stops
    log::info!("inserting stops...");
    let mut reader = csv::Reader::from_reader(File::open(path.join("stops.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_stop(client, row).await {
            report.skipped_stops += 1;
        }
        progress.inc();
    }
    progress.reset();

    // calendar
    log::info!("inserting calendar...");
    let mut reader = csv::Reader::from_reader(File::open(path.join("calendar.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_calendar_row(client, row).await {
            report.skipped_calendar_rows += 1;
        }
        progress.inc();
    }
    progress.reset();

    // calendar dates
    log::info!("inserting calendar dates...");
    let mut reader =
        csv::Reader::from_reader(File::open(path.join("calendar_dates.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_calendar_date(client, row).await {
            report.skipped_calendar_dates += 1;
        }
        progress.inc();
    }
    progress.reset();

    // trips
    log::info!("inserting trips...");
    let mut reader = csv::Reader::from_reader(File::open(path.join("trips.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_trip(client, row).await {
            report.skipped_trips += 1;
        }
        progress.inc();
    }
    progress.reset();

    // stop times
    log::info!("inserting stop times...");
    let mut reader =
        csv::Reader::from_reader(File::open(path.join("stop_times.txt"))?);
    for row in reader.deserialize() {
        if let Err(_) = insert_stop_time(client, row).await {
            report.skipped_stop_times += 1;
        }
        progress.inc();
    }

    Ok(report)
}

async fn insert_agency<D: Database>(
    client: &Client<D>,
    agency: Result<Agency, csv::Error>,
) -> Result<(), RequestError> {
    let agency = agency.map_err(RequestError::other)?;
    client
        .push_agency(
            model::agency::Agency {
                name: agency.name,
                website: agency.url,
                phone_number: agency.phone_number,
                email: agency.email,
                fare_url: agency.fare_url,
            },
            agency.id.clone().raw(),
        )
        .await?;
    Ok(())
}

async fn insert_route<D: Database>(
    client: &Client<D>,
    route: Result<Route, csv::Error>,
) -> Result<(), RequestError> {
    let route = route.map_err(RequestError::other)?;

    // TODO: exclude rail lines for now, as trip merging is not yet completely implemented.
    if matches!(route.kind, RouteType::Rail) {
        return Ok(());
    }

    let agency_id = if let Some(id) = route.agency_id {
        client.get_agency_id_by_original_id(id.raw()).await?
    } else {
        None
    };
    let name = route.long_name.or(route.short_name);
    client
        .push_line(
            model::line::Line {
                name,
                kind: match route.kind {
                    RouteType::TramStreetcarOrLighrail => {
                        LineType::TramStreetcarOrLighrail
                    }
                    RouteType::SubwayOrMetro => LineType::SubwayOrMetro,
                    RouteType::Rail => LineType::Rail,
                    RouteType::Bus => LineType::Bus,
                    RouteType::Ferry => LineType::Ferry,
                    RouteType::CableTram => LineType::CableTram,
                    RouteType::AerialLiftOrSuspendedCableCar => {
                        LineType::AerialLiftOrSuspendedCableCar
                    }
                    RouteType::Funicular => LineType::Funicular,
                    RouteType::Trolleybus => LineType::Trolleybus,
                    RouteType::Monorail => LineType::Monorail,
                },
                agency_id,
            },
            Some(route.id.raw()),
        )
        .await?;
    Ok(())
}

async fn insert_stop<D: Database>(
    client: &Client<D>,
    stop: Result<Stop, csv::Error>,
) -> Result<(), RequestError> {
    let stop = stop.map_err(RequestError::other)?;
    client
        .push_stop(
            model::stop::Stop {
                name: stop.name,
                description: stop.description,
                parent_id: None, // TODO!
                location: match (stop.latitude, stop.longitude) {
                    (Some(latitude), Some(longitude)) => {
                        Some(model::stop::Location {
                            latitude,
                            longitude,
                            address: None,
                        })
                    }
                    _ => None,
                },
                platform_code: stop.platform_code,
            },
            Some(stop.id.raw()),
        )
        .await?;
    Ok(())
}

async fn insert_calendar_row<D: Database>(
    client: &Client<D>,
    calender_row: Result<CalendarRow, csv::Error>,
) -> Result<(), RequestError> {
    let calendar_row = calender_row.map_err(RequestError::other)?;
    client
        .push_calendar_window(
            None,
            model::calendar::CalendarWindow {
                monday: calendar_row.monday.into(),
                tuesday: calendar_row.tuesday.into(),
                wednesday: calendar_row.wednesday.into(),
                thursday: calendar_row.thursday.into(),
                friday: calendar_row.friday.into(),
                saturday: calendar_row.saturday.into(),
                sunday: calendar_row.sunday.into(),
                start_date: calendar_row.start_date,
                end_date: calendar_row.end_date,
            },
            Some(calendar_row.service_id.raw()),
        )
        .await?;
    Ok(())
}

async fn insert_calendar_date<D: Database>(
    client: &Client<D>,
    calender_date: Result<CalendarDate, csv::Error>,
) -> Result<(), RequestError> {
    let calendar_date = calender_date.map_err(RequestError::other)?;
    let maybe_id = client
        .get_service_id_by_original_id(calendar_date.service_id.raw())
        .await
        .unwrap();
    client
        .push_calendar_date(
            maybe_id.as_ref(),
            model::calendar::CalendarDate {
                date: calendar_date.date,
                exception_type: calendar_date.exception_type.into(),
            },
            Some(calendar_date.service_id.raw()),
        )
        .await?;
    Ok(())
}

async fn insert_trip<D: Database>(
    client: &Client<D>,
    trip: Result<Trip, csv::Error>,
) -> Result<(), RequestError> {
    let trip = trip.map_err(RequestError::other)?;
    client
        .push_trip(
            model::trip::Trip {
                line_id: client
                    .get_line_id_by_original_id(trip.route_id.raw())
                    .await?
                    .ok_or(RequestError::IdMissing)?,
                service_id: client
                    .get_service_id_by_original_id(trip.service_id)
                    .await
                    .unwrap(),
                headsign: trip.headsign,
                short_name: trip.short_name,
                stops: vec![],
            },
            Some(trip.id.raw()),
            true,
        )
        .await?;
    Ok(())
}

async fn insert_stop_time<D: Database>(
    client: &Client<D>,
    stop_time: Result<StopTime, csv::Error>,
) -> Result<(), RequestError> {
    let stop_time = stop_time.map_err(RequestError::other)?;
    let stop_id = if let Some(orignal_stop_id) = stop_time.stop_id {
        client
            .get_stop_id_by_original_id(orignal_stop_id.raw())
            .await?
    } else {
        None
    };
    let trip_id = client
        .get_trip_id_by_original_id(stop_time.trip_id.raw())
        .await?
        .ok_or(RequestError::IdMissing)?;
    client
        .push_stop_time(
            trip_id,
            model::trip::StopTime {
                stop_sequence: stop_time.stop_sequence as i32,
                stop_id,
                arrival_time: stop_time.arrival_time,
                departure_time: stop_time.departure_time,
                stop_headsign: stop_time.stop_headsign,
            },
        )
        .await?;
    Ok(())
}
