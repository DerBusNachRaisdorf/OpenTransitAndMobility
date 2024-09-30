use std::{error::Error, fs::File, io::Read};

use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone};
use model::{
    trip_instance::TripInstance,
    trip_update::{
        StopTimeStatus, StopTimeUpdate, TripStatus, TripUpdate, TripUpdateId,
    },
    WithId,
};
use prost::Message;
use public_transport::{
    client::{instantiate_trip_naive, Client},
    database::Database,
    not_found_to_none, RequestError,
};
use utility::id::Id;

use crate::data_model::realtime::{
    self, trip_descriptor::ScheduleRelationship, trip_update::stop_time_update,
};

pub async fn update<D: Database>(
    client: Client<D>,
    url: &str,
) -> Result<Vec<WithId<TripUpdate>>, RequestError> {
    let response = reqwest::get(url)
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?;
    let message = realtime::FeedMessage::decode(&*bytes)
        .map_err(|why| RequestError::Other(Box::new(why)))?;

    let mut updates = vec![];
    for entity in message.entity {
        if let Some(trip_update) = entity.trip_update {
            // only care for updates with trip ids (for now)
            let original_trip_id = if let Some(id) = &trip_update.trip.trip_id {
                id
            } else {
                continue;
            };
            // get internal trip id
            let trip_id = if let Some(id) = client
                .get_trip_id_by_original_id(original_trip_id.clone())
                .await?
            {
                id
            } else {
                continue;
            };
            // only care for updates with trip start date (for now)
            let start_date = if let Some(date) = &trip_update.trip.start_date {
                NaiveDate::parse_from_str(date, "%Y%m%d")
                    .map_err(|why| RequestError::Other(Box::new(why)))?
            } else {
                continue;
            };

            // instanciate trip
            let trip = if let Some(trip) = not_found_to_none(
                client
                    .get_trip(trip_id.clone(), vec![client.origin()])
                    .await,
            )? {
                instantiate_trip_naive(&trip, &start_date, None, None)
            } else {
                None
            };

            // stop times
            let mut stop_times = vec![];
            for stop in trip_update.stop_time_update {
                let (arrival_time, departure_time) = get_times_for_stop(&trip, &stop);
                stop_times.push(StopTimeUpdate {
                    scheduled_stop_sequence: stop.stop_sequence.map(|i| i as i32),
                    arrival_time,
                    departure_time,
                    status: match stop.schedule_relationship() {
                        stop_time_update::ScheduleRelationship::NoData => {
                            StopTimeStatus::Unknown
                        }
                        stop_time_update::ScheduleRelationship::Scheduled => {
                            StopTimeStatus::Scheduled
                        }
                        stop_time_update::ScheduleRelationship::Skipped => {
                            StopTimeStatus::Cancelled
                        }
                        stop_time_update::ScheduleRelationship::Unscheduled => {
                            StopTimeStatus::Unknown // TODO!
                        }
                    },
                });
            }

            // Debug Print
            log::info!(
                "inserting update {}: {}",
                trip_id,
                stop_times
                    .iter()
                    .map(|st| format!(
                        "{}-{}",
                        st.arrival_time
                            .map(|t| t.to_string())
                            .unwrap_or("X".to_owned()),
                        st.departure_time
                            .map(|t| t.to_string())
                            .unwrap_or("X".to_owned())
                    ))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );

            let update = TripUpdate {
                status: match trip_update.trip.schedule_relationship() {
                    ScheduleRelationship::Added => TripStatus::Added,
                    ScheduleRelationship::Deleted => TripStatus::Deleted,
                    ScheduleRelationship::Canceled => TripStatus::Cancelled,
                    ScheduleRelationship::Scheduled => TripStatus::Scheduled,
                    ScheduleRelationship::Unscheduled => TripStatus::Unscheduled,
                    // TODO...
                    _ => {
                        continue;
                    }
                },
                stops: stop_times,
                timestamp: trip_update
                    .timestamp
                    .and_then(|ts| Local.timestamp_opt(ts as i64, 0).earliest()),
            };

            updates.push(WithId::new(
                Id::new(TripUpdateId::new(trip_id, start_date)),
                update,
            ));
        }
        // TODO: service alerts...
    }

    client.put_trip_updates(updates).await
}

fn get_times_for_stop(
    trip: &Option<TripInstance>,
    stop: &crate::data_model::realtime::trip_update::StopTimeUpdate,
) -> (Option<DateTime<Local>>, Option<DateTime<Local>>) {
    let arrival_time = stop.arrival.as_ref().and_then(|arrival| arrival.time);
    let arrival_delay = stop.arrival.as_ref().and_then(|arrival| arrival.delay);
    let departure_time = stop.departure.as_ref().and_then(|departure| departure.time);
    let departure_delay = stop
        .departure
        .as_ref()
        .and_then(|departure| departure.delay);

    let arrival = match (arrival_time, arrival_delay, &trip, stop.stop_sequence) {
        (Some(time), _, _, _) => Local.timestamp_opt(time, 0).earliest(),
        (None, Some(delay), Some(trip), Some(seq)) => trip
            .get_stop_time_by_sequence(seq as i32)
            .and_then(|stop_time| stop_time.arrival_time)
            .map(|time| time + Duration::seconds(delay as i64)),
        _ => None,
    };

    let departure = match (departure_time, departure_delay, &trip, stop.stop_sequence)
    {
        (Some(time), _, _, _) => Local.timestamp_opt(time, 0).earliest(),
        (None, Some(delay), Some(trip), Some(seq)) => trip
            .get_stop_time_by_sequence(seq as i32)
            .and_then(|stop_time| stop_time.departure_time)
            .map(|time| time + Duration::seconds(delay as i64)),
        _ => None,
    };

    (arrival, departure)
}

// DEBUG

pub fn test() -> Result<(), Box<dyn Error>> {
    let file_path = "resources/realtime-free.pb";
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let message = realtime::FeedMessage::decode(&buffer[..])?;
    for entity in &message.entity {
        if let Some(vehicle) = &entity.vehicle {
            let id = vehicle
                .trip
                .as_ref()
                .and_then(|trip| trip.trip_id.clone())
                .unwrap_or("<unknown>".to_owned());
            let position = vehicle
                .position
                .as_ref()
                .map(|position| {
                    format!("({}, {})", position.latitude, position.longitude)
                })
                .unwrap_or("<unknown>".to_string());
            println!("-> {}: {}", id, position);
        }
        if let Some(_trip_update) = &entity.trip_update {
            // TODO...
        }
    }
    dbg!(message);
    Ok(())
}
