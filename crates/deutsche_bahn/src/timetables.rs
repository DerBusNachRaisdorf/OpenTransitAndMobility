use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Duration, Local};

use super::{ApiError, STATION_TABLE};
use crate::client::Accept;
use crate::make_valid_station_name_key;
use crate::{client::BahnApiClient, model::timetables::*};

/* - BAHN TIMETABLES API */

#[derive(Debug, Serialize, Deserialize)]
pub struct Stations {
    #[serde(alias = "$value", rename = "stations", default)]
    pub value: Vec<StationData>,
}

pub async fn get_stations(
    client: Arc<BahnApiClient>,
    pattern: &str,
) -> Result<Stations, ApiError> {
    /* translate problemtic stations */
    let station_pattern = match STATION_TABLE.get(&pattern.to_lowercase()) {
        Some(x) => x,
        None => pattern,
    };

    /* fetch data */
    client
        .get(
            &format!("timetables/v1/station/{station_pattern}"),
            Accept::Xml,
        )
        .await
}

trait TimetableStopUpdate {
    fn timetablestop_update(&mut self, _other: &Self) -> bool;
}

impl TimetableStopUpdate for String {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        if *self != *other {
            *self = other.clone();
            true
        } else {
            false
        }
    }
}

impl TimetableStopUpdate for i64 {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        if *self != *other {
            *self = *other;
            true
        } else {
            false
        }
    }
}

impl TimetableStopUpdate for DateTime<Local> {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        if *self == *other {
            false
        } else {
            let res = if (*other - *self).num_minutes() > 1 {
                true
            } else {
                false
            };
            *self = other.clone();
            res
        }
    }
}

impl<T: TimetableStopUpdate + Clone> TimetableStopUpdate for Option<T> {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        if let Some(other_val) = other.clone() {
            if let Some(mut self_val) = self.clone() {
                let res = self_val.timetablestop_update(&other_val);
                *self = Some(self_val);
                res
            } else {
                *self = other.clone();
                true
            }
        } else {
            false
        }
    }
}

impl<T: Clone> TimetableStopUpdate for Vec<T> {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        // TODO: better?
        if self.len() > other.len() {
            *self = other.clone();
            true
        } else {
            false
        }
    }
}

impl TimetableStopUpdate for EventStatus {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        if *self != *other {
            *self = other.clone();
            true
        } else {
            false
        }
    }
}

impl TimetableStopUpdate for Event {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        let mut res = false;
        /*res |= */
        self.changed_distant_endpoint
            .timetablestop_update(&other.changed_distant_endpoint);
        self.cancellation_time
            .timetablestop_update(&other.cancellation_time);

        let has_platform_changed = self
            .changed_platform
            .timetablestop_update(&other.changed_platform);
        if has_platform_changed {
            match (&self.changed_platform, &self.planned_platform) {
                (Some(a), Some(b)) => {
                    res |= a != b;
                }
                _ => {}
            }
        }

        /* path was changed... */
        if self.changed_path.timetablestop_update(&other.changed_path) {
            res = true;
            /* recaclulate the actual path */
            self.calculate_actual_path();
        }

        /* status was changed... */
        if self
            .changed_status
            .timetablestop_update(&other.changed_status)
        {
            self.calculate_actual_status();
            res |= self.actual_status == EventStatus::Cancelled;
        }

        let mut has_changed_time =
            self.changed_time.timetablestop_update(&other.changed_time);
        if let Some(planned_time) = self.planned_time {
            if let Some(changed_time) = self.changed_time {
                has_changed_time &= (changed_time - planned_time).num_minutes() >= 5;
            }
        }
        res |= has_changed_time;

        self.distant_change
            .timetablestop_update(&other.distant_change);
        res
    }
}

impl TimetableStopUpdate for TimetableStop {
    fn timetablestop_update(&mut self, other: &Self) -> bool {
        let mut result = false;
        result |= self.arrival.timetablestop_update(&other.arrival);
        result |= self.departure.timetablestop_update(&other.departure);
        // TODO: others!
        result
    }
}

/// timetables GET /fchg/{evaNo}
///
/// DESCRIPTION (https://developers.deutschebahn.com/db-api-marketplace/apis/product/timetables/api/26494#/Timetables_10213/operation/%2Ffchg%2F{evaNo}/get):
/// Returns a Timetable object (see Timetable) that contains all known changes for the station given by evaNo.
///
/// The data includes all known changes from now on until ndefinitely into the future.
/// Once changes become obsolete (because their trip departs from the station) they are removed from this resource.
///
/// Changes may include messages.
/// On event level, they usually contain one or more of the 'changed' attributes ct, cp, cs or cpth.
/// Changes may also include 'planned' attributes if there is no associated planned data for the change (e.g. an unplanned stop or trip).
///
/// Full changes are updated every 30s and should be cached for that period by web caches.
pub async fn get_known_changes(
    client: &BahnApiClient,
    eva: i64,
) -> Result<Timetable, ApiError> {
    client
        .get(&format!("timetables/v1/fchg/{eva}"), Accept::Xml)
        .await
}

/// Returns a Timetable object (see Timetable) that contains all recent changes for the station given by evaNo.
/// Recent changes are always a subset of the full changes. They may equal full changes but are typically much smaller.
/// Data includes only those changes that became known within the last 2 minutes.
///
/// A client that updates its state in intervals of less than 2 minutes should load full changes initially and then
/// proceed to periodically load only the recent changes in order to save bandwidth.
///
/// Recent changes are updated every 30s as well and should be cached for that period by web caches.
pub async fn get_recent_changes(
    client: &BahnApiClient,
    eva: i64,
) -> Result<Timetable, ApiError> {
    client
        .get(&format!("timetables/v1/rchg/{eva}"), Accept::Xml)
        .await
}

/// Returns a Timetable object (see Timetable) that contains planned data for the
/// specified station (evaNo) within the hourly time slice given by date (format YYMMDD)
/// and hour (format HH). The data includes stops for all trips that arrive or depart within that slice.
/// There is a small overlap between slices since some trips arrive in one slice and depart in another.
///
/// Planned data does never contain messages. On event level, planned data contains the 'planned'
/// attributes pt, pp, ps and ppth while the 'changed' attributes ct, cp, cs and cpth are absent.
///
/// Planned data is generated many hours in advance and is static, i.e. it does never change.
/// It should be cached by web caches.public interface allows access to information about a station.
pub async fn get_plan(
    client: &BahnApiClient,
    eva: i64,
    time: DateTime<Local>,
) -> Result<Timetable, ApiError> {
    let date_str = time.format("%y%m%d");
    let hour_str = time.format("%H");
    println!("timetables/v1/plan/{eva}/{date_str}/{hour_str}");

    /* http GET request */
    client
        .get(
            &format!("timetables/v1/plan/{eva}/{date_str}/{hour_str}"),
            Accept::Xml,
        )
        .await
}

/* -- NEWS -- */

/// For how many hours to fetch plan-data.
const TIMETABLE_NEWS_PREFETCH: i64 = 2;

/// Minimum update interval in Minutes.
const TIMETABLE_UPDATE_INTERVAL: i64 = 2;

/// After how many minutes to remove an outdated stop.
const REMOVE_STOP_AFTER: i64 = 120;

pub enum UpdateResult<U, E> {
    Ok(U),
    OkNoValues,
    ErrButValues(U, E),
    Err(E),
}

impl<U, E> UpdateResult<U, E> {
    pub fn is_error(&self) -> bool {
        match self {
            Self::Err(_) | Self::ErrButValues(_, _) => true,
            _ => false,
        }
    }
}

/// Checks whether an event is outdated.
/// minutes_tolerance specifies how many minutes an event must to be outdated,
///     to also be considered outdated by this function.
fn is_event_outdated(event: &Event, minutes_tolerance: i64) -> bool {
    let now = chrono::offset::Local::now() - Duration::minutes(minutes_tolerance);
    if let Some(planned_time) = event.planned_time {
        if planned_time >= now {
            return false;
        }
    }
    if let Some(changed_time) = event.changed_time {
        if changed_time >= now {
            return false;
        }
    }
    true
}

/// Same as is_event_outdated but for a stop.
fn is_stop_outdated(stop: &TimetableStop, minutes_tolerance: i64) -> bool {
    let is_arrival_outdated = stop
        .arrival
        .as_ref()
        .map(|arrival| is_event_outdated(arrival, minutes_tolerance))
        .unwrap_or(true);
    let is_departure_outdated = stop
        .departure
        .as_ref()
        .map(|departure| is_event_outdated(departure, minutes_tolerance))
        .unwrap_or(true);
    is_arrival_outdated && is_departure_outdated
}

pub struct TimetableNews {
    bahn_api_client: Arc<BahnApiClient>,
    eva: i64,
    stops: RwLock<HashMap<String, Arc<RwLock<TimetableStop>>>>,
    fetch_next: RwLock<DateTime<Local>>,
    last_outdated_removed: RwLock<DateTime<Local>>,
    last_update: RwLock<Option<DateTime<Local>>>,
    station_name: String,
    station_name_aliases: Vec<String>,
    removed_stops: RwLock<Vec<TimetableStop>>,
    unapplied_known_changes_cache: RwLock<Vec<TimetableStop>>,
}

impl TimetableNews {
    pub async fn new(
        bahn_api_client: Arc<BahnApiClient>,
        station_pattern: &str,
        name_aliases: Vec<String>,
        _ignore_known_at_launch: bool,
    ) -> Result<Self, ApiError> {
        let station = get_stations(bahn_api_client.clone(), station_pattern)
            .await?
            .value
            .first()
            .ok_or(ApiError::StationDoesNotExist(station_pattern.to_owned()))?
            .clone();

        let result = Self {
            bahn_api_client: bahn_api_client.clone(),
            eva: station.eva,
            stops: RwLock::new(HashMap::new()),
            fetch_next: RwLock::new(chrono::offset::Local::now()),
            last_outdated_removed: RwLock::new(chrono::offset::Local::now()),
            last_update: RwLock::new(None),
            station_name: station.name.clone(),
            station_name_aliases: name_aliases,
            removed_stops: RwLock::new(Vec::new()),
            unapplied_known_changes_cache: RwLock::new(Vec::new()),
        };

        Ok(result)
    }

    pub async fn live_data_last_updated_at(&self) -> Option<DateTime<Local>> {
        *self.last_update.read().await
    }

    /// Updates the Timetable and - if successfull - returns a vec of all updated station
    /// timetables.
    pub async fn get_updates(
        &self,
    ) -> UpdateResult<Vec<Arc<RwLock<TimetableStop>>>, Vec<ApiError>> {
        match *self.last_update.read().await {
            Some(date) => {
                if (chrono::offset::Local::now() - date).num_minutes()
                    < TIMETABLE_UPDATE_INTERVAL
                {
                    // cache last update result
                    // better: only fetch plan data
                    //todo!()
                }
            }
            _ => {}
        }

        let fetch_plan_result = self.fetch_plan().await;
        let get_known_changes_result = self.get_known_changes().await;

        let result = match (fetch_plan_result, get_known_changes_result) {
            /* update if changes where successfully fetched (plan data maybe not) */
            (
                UpdateResult::Err(why) | UpdateResult::ErrButValues(_, why),
                Ok(timetable),
            ) => {
                let values = self.apply_updates(timetable.stops).await;
                if values.is_empty() {
                    UpdateResult::Err(vec![why])
                } else {
                    UpdateResult::ErrButValues(values, vec![why])
                }
            }
            (_, Ok(timetable)) => {
                let values = self.apply_updates(timetable.stops).await;
                if values.is_empty() {
                    UpdateResult::OkNoValues
                } else {
                    UpdateResult::Ok(values)
                }
            }
            /* update from cache if changes could not be fetched, but plan data changed */
            (UpdateResult::Ok(_) | UpdateResult::OkNoValues, Err(why)) => {
                let cached_updates = self
                    .unapplied_known_changes_cache
                    .write()
                    .await
                    .drain(..)
                    .collect();
                UpdateResult::ErrButValues(
                    self.apply_updates(cached_updates).await,
                    vec![why],
                )
            }
            (UpdateResult::ErrButValues(_, why1), Err(why2)) => {
                let cached_updates = self
                    .unapplied_known_changes_cache
                    .write()
                    .await
                    .drain(..)
                    .collect();
                UpdateResult::ErrButValues(
                    self.apply_updates(cached_updates).await,
                    vec![why1, why2],
                )
            }
            /* nothing to update, only errors */
            (UpdateResult::Err(why1), Err(why2)) => {
                UpdateResult::Err(vec![why1, why2])
            }
        };

        /* remove outdated */
        let now = chrono::offset::Local::now();
        {
            if (now - *self.last_outdated_removed.read().await).num_hours() >= 2 {
                self.remove_outdated().await;
            }
        }

        result
    }

    /// fetches updates from the Bahn-API.
    /// Sets the `last_update` to the current time if successful.
    async fn get_known_changes(&self) -> Result<Timetable, ApiError> {
        match get_known_changes(&self.bahn_api_client, self.eva).await {
            Ok(res) => {
                *self.last_update.write().await = Some(chrono::offset::Local::now());
                Ok(res)
            }
            Err(why) => Err(why),
        }
    }

    /// Applies the given vec of updates.
    async fn apply_updates(
        &self,
        changes: Vec<TimetableStop>,
    ) -> Vec<Arc<RwLock<TimetableStop>>> {
        let mut result: Vec<Arc<RwLock<TimetableStop>>> = Vec::new();

        // stores all updates that couldn't be applied due the stop not being known at the current
        // time. (Most likely because the plan data for this stop wasn't fetched already)
        let mut cache_updates = Vec::<TimetableStop>::new();

        {
            let mut stops_map = self.stops.write().await;
            for mut stop_change in changes {
                /* is change for a known trip? ... */
                if let Some(stop) =
                    stops_map.get_mut(&stop_change.id.full_id_string())
                {
                    if stop.write().await.timetablestop_update(&stop_change) {
                        result.push(stop.clone());
                        /*
                        println!("Updated stop: {} ->\n{} ->\n{}",
                            &stop.read().await.id.full_id_string(),
                            serde_json::to_string_pretty(&stop_change).unwrap(),
                            serde_json::to_string_pretty(&*stop.read().await).unwrap()
                        );
                        */
                    }
                /* ...or is it for an unknown trip but added? */
                } else if stop_change.is_added() {
                    let id = stop_change.id.full_id_string();
                    stop_change.calculate_all();
                    let added_stop = Arc::new(RwLock::new(stop_change));
                    stops_map.insert(id, added_stop.clone());
                    result.push(added_stop);
                /* ...or is it for an unknown trip */
                } else {
                    /* cache update */
                    cache_updates.push(stop_change);
                }
            }
        }

        // overwrite cached updates
        *self.unapplied_known_changes_cache.write().await = cache_updates;

        result
    }

    pub(crate) async fn get_stops_internal(
        &self,
    ) -> Result<Vec<Arc<RwLock<TimetableStop>>>, ApiError> {
        let stops: Vec<Arc<RwLock<TimetableStop>>> = self
            .stops
            .read()
            .await
            .values()
            .map(|x| x.clone())
            .collect();
        Ok(stops)
    }

    /// Get all current stops in chronological order.
    /// Does NOT perform an update.
    pub async fn get_current(&self) -> Result<Timetable, ApiError> {
        self.remove_outdated().await;

        // TODO: Save current stops in an ordered Vec<Arc<RwLock<TimetableStop>>>

        let mut current_stops: Vec<_> = futures::future::join_all(
            self.stops
                .read()
                .await
                .values()
                .map(|x| async { x.read().await.clone() }),
        )
        .await;

        /* filter outdated stops since remove_outated only removes VERY outdated stops. */
        current_stops = current_stops
            .into_iter()
            .filter(|stop| !is_stop_outdated(stop, 2))
            .collect();

        /* sort stops by departure/arrival times */
        current_stops.sort_by(|a, b| -> std::cmp::Ordering {
            // Returns the fist of the following that is not none:
            // 1. departure planned time
            // 2. departure changed time
            // 3. arrival planned time
            // 4. arrival changed time
            let get_most_significant_time: fn(&TimetableStop) -> DateTime<Local> =
                |stop| {
                    let default_time = chrono::offset::Local::now();
                    stop.departure
                        .as_ref()
                        .or(stop.arrival.as_ref())
                        .map(|event| {
                            event
                                .planned_time
                                .or(event.changed_time)
                                .unwrap_or(default_time)
                        })
                        .unwrap_or(default_time)
                };

            /* compare stop a and b */
            get_most_significant_time(a)
                .partial_cmp(&get_most_significant_time(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(Timetable {
            eva: Some(self.eva),
            stops: current_stops,
            station_name: Some(self.station_name.clone()),
            messages: vec![],
            live_data_last_updated_at: *self.last_update.read().await,
        })
    }

    /// Fetches all missing plan data.
    async fn fetch_plan(&self) -> UpdateResult<(), ApiError> {
        let current_time = chrono::offset::Local::now();
        let mut new_stops = Vec::<TimetableStop>::new();

        let mut result = UpdateResult::OkNoValues;
        {
            let mut fetch_next = self.fetch_next.write().await;
            while *fetch_next
                < current_time + chrono::Duration::hours(TIMETABLE_NEWS_PREFETCH)
            {
                match get_plan(&self.bahn_api_client, self.eva, *fetch_next).await {
                    Ok(mut o) => {
                        new_stops.append(&mut o.stops);
                        *fetch_next += chrono::Duration::hours(1);
                        result = UpdateResult::Ok(());
                    }
                    Err(why) => {
                        if let UpdateResult::Ok(_) = result {
                            result = UpdateResult::ErrButValues((), why);
                        } else {
                            result = UpdateResult::Err(why);
                        }
                        break;
                    }
                }
            }
        }

        let mut stops_map = self.stops.write().await;
        for mut stop in new_stops {
            /* calculate the actual path */
            stop.calculate_all();
            stops_map.insert(stop.id.full_id_string(), Arc::new(RwLock::new(stop)));
        }

        result
    }

    /// Removes all stops that are very outdated.
    async fn remove_outdated(&self) {
        let now = chrono::offset::Local::now();

        let mut remove = Vec::<String>::new();
        {
            let stops = self.stops.read().await;
            for (id, stop) in stops.iter() {
                let mut is_outdated = true;
                /* check if outdated */
                if let Some(arrival) = &stop.read().await.arrival {
                    if !is_event_outdated(&arrival, REMOVE_STOP_AFTER) {
                        is_outdated = false;
                    }
                }
                if let Some(departure) = &stop.read().await.departure {
                    if !is_event_outdated(&departure, REMOVE_STOP_AFTER) {
                        is_outdated = false;
                    }
                }

                /* remember to remove */
                if is_outdated {
                    remove.push(id.clone());
                }
            }
        }

        let mut removed_stops: Vec<TimetableStop> = Vec::new();
        {
            let mut stops = self.stops.write().await;
            for id in remove {
                let stop_opt = stops.remove(&id);
                if let Some(stop) = stop_opt {
                    removed_stops.push(stop.read().await.clone());
                }
                //println!("Removed outdated stop: {}", id);
            }
        }

        self.removed_stops.write().await.append(&mut removed_stops);

        *self.last_outdated_removed.write().await = now;
    }

    /// Gets all removed stops and clears the removed stops cache.
    /// When stops are removed, they get cached until this function is called.
    pub async fn get_and_clear_removed_stops(&self) -> Vec<TimetableStop> {
        let mut removed_stops = self.removed_stops.write().await;
        let res = removed_stops.clone();
        removed_stops.clear();
        res
    }

    pub fn station_name(&self) -> String {
        self.station_name.clone()
    }

    pub fn is_own_station_name(&self, name: &str) -> bool {
        name == self.station_name()
            || self.station_name_aliases.contains(&name.to_owned())
            || make_valid_station_name_key(name)
                == make_valid_station_name_key(&self.station_name())
    }

    pub fn eva(&self) -> i64 {
        self.eva
    }
}
