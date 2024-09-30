use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use crate::{make_valid_station_name_key, ApiError};
use crate::client::BahnApiClient;
use crate::timetables::*;
use crate::model::timetables::*;

// TODO list
// [x] update all timetables even if not starting at index 0. maybe already done, cant remember

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripStop {
    info: ActualPathStop,
    stop: Option<TimetableStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    id: String,
    line: String,
    category: String,
    stops: Vec<TripStop>,
}

#[derive(Debug, Clone)]
struct InternalTripStop {
    info: ActualPathStop,
    stop: Option<Arc<RwLock<TimetableStop>>>,
}

impl InternalTripStop {
    async fn trip_stop(&self) -> TripStop {
        let cloned_stop = if let Some(stop) = &self.stop {
            Some(stop.read().await.clone())
        } else {
            None
        };
        TripStop {
            info: self.info.clone(),
            stop: cloned_stop,
        }
    }
}

#[derive(Debug, Clone)]
struct InternalTrip {
    id: String,
    line: String,
    category: String,
    stops: Vec<InternalTripStop>,
    last_updated: Option<chrono::DateTime<chrono::Local>>,
}

impl InternalTrip {
    async fn trip(&self) -> Trip {
        Trip {
            id: self.id.clone(),
            line: self.line.clone(),
            category: self.category.clone(),
            stops: join_all(self.stops.iter().map(|s| async { s.trip_stop().await })).await
        }
    }
}

pub struct Triptable {
    /// key: void station name key
    timetables: RwLock<HashMap<String, Arc<TimetableNews>>>,

    /// key: trip-id (daily_trip_id - date_specifier)
    trips: RwLock<HashMap<String, Arc<RwLock<InternalTrip>>>>,

    /// might be obsolete
    add_stations_queue: RwLock<Vec<(String, Vec<String>, String, Arc<BahnApiClient>)>>,

    timetables_update_queue: RwLock<Vec<Arc<TimetableNews>>>,
}

impl Triptable {
    pub async fn new() -> Result<Self, ApiError> {
        let result = Self {
            timetables: RwLock::new(HashMap::new()),
            trips: RwLock::new(HashMap::new()),
            add_stations_queue: RwLock::new(Vec::new()),
            timetables_update_queue: RwLock::new(Vec::new()),
        };
        result.update().await?;
        Ok(result)
    }

    #[allow(dead_code)]
    async fn get_internal_trip(&self, trip_id: &str) -> Option<Arc<RwLock<InternalTrip>>> {
        self.trips.read().await.get(trip_id).cloned()
    }

    pub async fn get_trip(&self, trip_id: &str) -> Option<Trip> {
        if let Some(trip) = self.trips.read().await.get(trip_id) {
            Some(trip.read().await.trip().await)
        } else {
            None
        }
    }

    pub async fn get_station_timetable(&self, name: &str) -> Option<Arc<TimetableNews>> {
        self.timetables.read().await.get(name).cloned() 
    }

    async fn try_add_station(
        &self,
        name: &str,
        name_aliases: Vec<String>,
        pattern: &str,
        bahn_api_client: Arc<BahnApiClient>
    ) -> Result<(), ApiError> {
        let timetable = Arc::new(
            TimetableNews::new(
                bahn_api_client,
                pattern,
                name_aliases,
                true,
            ).await?
        );
        let key = make_valid_station_name_key(name);
        self.timetables.write().await.insert(key, timetable.clone());
        self.timetables_update_queue.write().await.push(timetable);
        Ok(())
    }

    pub async fn add_station(
        &self,
        name: &str,
        name_aliases: Vec<String>,
        pattern: &str, // TODO: take StaDa-Entry instead, also save stada entry in TimetableNews
        bahn_api_client: Arc<BahnApiClient>,
    ) {
        if let Err(why) = self.try_add_station(name, name_aliases.clone(), pattern, bahn_api_client.clone()).await {
            // TODO: vernünftiges logging system, das ist ja gruselig hier
            match why{
                ApiError::StationDoesNotExist(s) => {
                    println!("Station '{}' does not exist. Not adding.", s);
                },
                _ => {
                    self.add_stations_queue
                        .write()
                        .await
                        .push((name.to_owned(), name_aliases, pattern.to_owned(), bahn_api_client));
                    println!("[TripTable]: Could not add Station '{}' -> added to queue: {:?}", pattern, why);
                    if !matches!(why, ApiError::RateLimitReached) {
                        println!("Cout not add station '{}': {}.", pattern, why);
                    }
                },
            }
        }
    }

    pub async fn add_stations(
        &self,
        stations: Vec<(String, Vec<String>, String, Arc<BahnApiClient>)>, // TODO: same as add_station regarding pattern -> StaDa-Entry
    ) {
        let mut add_to_queue = Vec::new();
        for (name, name_aliases, pattern, bahn_api_client) in stations {
            if let Err(why) = self.try_add_station(&name, name_aliases.clone(), &pattern, bahn_api_client.clone()).await {
                match why{
                    ApiError::StationDoesNotExist(s) => {
                        println!("Station '{}' does not exist. Not adding.", s);
                    },
                    _ => {
                        add_to_queue.push((name.to_owned(), name_aliases, pattern.to_owned(), bahn_api_client.clone()));
                        if !matches!(why, ApiError::RateLimitReached) {
                            println!("Cout not add station '{}': {}.", pattern, why);
                        }
                    },
                }
            }
        }
        self.add_stations_queue.write().await.append(&mut add_to_queue);
    }

    pub async fn update(&self) -> Result<(HashMap<String, (String, Vec<Arc<RwLock<TimetableStop>>>)>, Vec<(String, Vec<TimetableStop>)>), ApiError> {
        /* first add queued-to-add stations, they should not be starved */
        let add_stations_queue = self.add_stations_queue
            .write()
            .await
            .drain(..)
            .collect::<Vec<_>>();
        self.add_stations(add_stations_queue).await;

        // TODO: this method must also return all successful updates if one failed.
        println!("- TRIPTABLE UPDATE -");
        let mut stations_updates = HashMap::<String, (String, Vec<Arc<RwLock<TimetableStop>>>)>::new();
        let mut stations_removed_stops = Vec::<(String, Vec<TimetableStop>)>::new();

        /* alle timetables durchgehen */
        let mut queue_prio_next = Vec::<Arc<TimetableNews>>::new();
        let mut queue_not_prio_next= Vec::<Arc<TimetableNews>>::new();
        for timetable in self.timetables_update_queue.read().await.iter() {
            /* insert removed stops */
            stations_removed_stops.push((timetable.station_name(), timetable.get_and_clear_removed_stops().await));
            println!("updatating timetable {}", timetable.station_name());

            /* update timetable */
            let updates = match timetable.get_updates().await {
                UpdateResult::Ok(u) => {
                    // successful update => don't priorize this station next update
                    queue_not_prio_next.push(timetable.clone());
                    u
                },
                UpdateResult::OkNoValues => {
                    // successful update => don't priorize this station next update
                    queue_not_prio_next.push(timetable.clone());
                    vec![]
                },
                UpdateResult::ErrButValues(u, whys) => {
                    // partial successful update => priorize this station next update
                    let whys_str = whys.iter()
                        .map(|why| why.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!(
                        "Cannot update timetable for station '{}': {}",
                        timetable.station_name(),
                        whys_str,
                    );
                    queue_prio_next.push(timetable.clone());
                    u
                },
                UpdateResult::Err(whys) => {
                    // unsuccessful update => priorize this station next update
                    let whys_str = whys.iter()
                        .map(|why| why.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!(
                        "Cannot update timetable for station '{}': {}.",
                        timetable.station_name(),
                        whys_str,
                    );
                    queue_prio_next.push(timetable.clone());
                    vec![]
                },
            }.clone(); 
            let timetable_live_data_last_updated_at = timetable.live_data_last_updated_at().await;
            stations_updates.insert(make_valid_station_name_key(&timetable.station_name()), (timetable.station_name(), updates));
            /* get current stops */
            let current_stops = timetable.get_stops_internal().await?.clone();
            for stop in current_stops {
                // TODO! die nächsten vier rwlock.read() Aufrufe lassen sich auf einen reduzieren,
                // aber Faulheit kickt gerade anders.
                let trip_id = stop.read().await.id.trip_id_string();
                let line = if let Some(arrival) = &stop.read().await.arrival {
                    arrival.line.clone().unwrap_or("unkown".to_owned())
                } else if let Some(departure) = &stop.read().await.departure {
                    departure.line.clone().unwrap_or("unkown".to_owned())
                } else {
                    "unkown".to_owned()
                };
                let category = stop.read().await.trip_label.clone()
                    .map(|lbl| lbl.category)
                    .unwrap_or("unknown".to_string());

                /* if trip is alread in trips, => update it... */
                if self.trips.read().await.contains_key(&trip_id) {
                    let trip = self.trips.read().await.get(&trip_id).unwrap().clone();
                    let trip_last_updated_at = trip.read().await.last_updated;
                    let should_update = match (timetable_live_data_last_updated_at, trip_last_updated_at) {
                        (Some(date1), Some(date2)) => date1 >= date2,
                        (Some(_), None) => true,
                        _ => false,
                    };
                    println!("stop for trip '{} ({})' of timetable '{}' is already in trip-table",
                        line.clone(), line.clone(), timetable.station_name());
                    /* update arrival path */
                    let mut i = 0usize;
                    /* build actual path WITH own stop */
                    let mut actual_path = stop.read().await.arrival_path().cloned().unwrap_or(vec![]);
                    actual_path.push(ActualPathStop { status: stop.read().await.status(), name: timetable.station_name() });
                    actual_path.append(&mut stop.read().await.departure_path().cloned().unwrap_or(vec![]));
                    /* update trips path */
                    for actual_path_stop in actual_path /*stop.read().await.create_full_path_without_own_stop()*/ {
                        // TODO: handle trips with multiple stops at the same station
                        //       (very rare, so im going to ignore it for now)
                        let stops_len = trip.read().await.stops.len();
                        /* check if 'actual_path_stop' is at 'timetable's station and has not been
                         * set yet... */
                        if i < stops_len {
                            let is_timetable_station_and_not_set = {
                                let trip_stop_at_i = &trip.read().await.stops[i];
                                if trip_stop_at_i.stop.is_none() {
                                    //trip_stop_at_i.info.name == timetable.station_name()
                                    timetable.is_own_station_name(&trip_stop_at_i.info.name)
                                } else {
                                    false
                                }
                            };
                            /* if 'actual_path_stop' is at 'timetable's station and has not been set
                            * yet, => set it! */
                            if is_timetable_station_and_not_set {
                                trip.write().await.stops[i].stop = Some(stop.clone());
                            }
                        }

                        /* ...else if both stops are the same => update! */
                        if i < stops_len && actual_path_stop.name == trip.read().await.stops[i].info.name {
                            if should_update {
                                if actual_path_stop.status != trip.read().await.stops[i].info.status {
                                    trip.write().await.stops[i].info.status = actual_path_stop.status.clone();
                                }
                                trip.write().await.last_updated = timetable_live_data_last_updated_at;
                            }
                            i += 1;
                        /* ...otherwise, the stop is not yet in the trip => insert it! */
                        } else if should_update {
                            if i > stops_len {
                                let trip_id = &trip.read().await.id;
                                println!("Cannot update stops of trip '{}'. Index `i` is too big (is `{}`, should be `<= {}`). ***This is a bug*** and should *not* occur.", trip_id, i, stops_len);
                                break;
                            }
                            trip.write().await.stops.insert(i, InternalTripStop {
                                info: actual_path_stop.clone(),
                                stop: if actual_path_stop.name == timetable.station_name() { Some(stop.clone()) } else { None },
                            });
                            trip.write().await.last_updated = timetable_live_data_last_updated_at;
                            i += 1;
                        }
                    }
                /* ...else the trip is not already in trips => insert it */
                } else {
                    println!("stop for trip '{} ({})' of timetable '{}' not yet in trip-table",
                        line.clone(), trip_id.clone(), timetable.station_name());

                    println!("|-inserting arrival-path...");
                    let mut trip_stops: Vec<InternalTripStop> = stop.read().await
                        .arrival_path()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(|actual_path_stop| {
                            InternalTripStop {
                                info: actual_path_stop.clone(),
                                stop: None,
                            }
                        })
                        .collect();

                    println!("|-inserting own stop...");
                    trip_stops.push(InternalTripStop { 
                        info: ActualPathStop {
                            status: stop.read().await.status(),
                            name: timetable.station_name(),
                        },
                        stop: Some(stop.clone()),
                    });

                    println!("|-inserting departure-path...");
                    trip_stops.append(
                        &mut stop.read().await
                            .departure_path()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .map(|actual_path_stop| {
                                InternalTripStop {
                                    info: actual_path_stop.clone(),
                                    stop: None,
                                }
                            })
                            .collect()
                    );

                    println!("|-inserting trip into trips...");
                    self.trips.write().await.insert(trip_id.clone(), Arc::new(RwLock::new(InternalTrip {
                            id: trip_id,
                            line,
                            category,
                            stops: trip_stops,
                            last_updated: timetable_live_data_last_updated_at,
                        }
                    )));

                    println!("|- DONE!");
                }
            }
        }
        /* update update queue */
        queue_prio_next.append(&mut queue_not_prio_next);
        *self.timetables_update_queue.write().await = queue_prio_next;

        Ok((stations_updates, stations_removed_stops))
    }
}
