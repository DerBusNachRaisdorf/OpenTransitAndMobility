use std::collections::HashMap;

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};
use model::{
    agency::Agency,
    calendar::{CalendarDate, CalendarWindow, Service},
    filter_sort_subjects,
    line::Line,
    merge_all_from,
    origin::Origin,
    shared_mobility::{SharedMobilityStation, Status},
    stop::{Stop, StopNameSuggestion},
    trip::{StopTime, Trip},
    trip_instance::{StopTimeInstance, TripInstance, TripInstanceInfo},
    trip_update::{StopTimeUpdate, TripStatus, TripUpdate, TripUpdateId},
    DatabaseEntry, DatabaseEntryCollection, DateTimeRange, Mergable, WithDistance,
    WithId, WithOrigin,
};
use serde::Serialize;
use utility::{id::Id, let_also::LetAlso};

use crate::{
    database::{
        AgencyRepo, Database, DatabaseOperations, DatabaseTransaction, LineRepo,
        MergableRepo, RealtimeRepo, Repo, ServiceRepo, SharedMobilityStationRepo,
        StopRepo, SubjectRepo, TripRepo,
    },
    not_found_to_none, RequestError, RequestResult,
};

#[derive(Debug, Clone)]
pub enum Update {
    TripUpdate { origin: Id<Origin>, id: Id<Trip> },
}

#[derive(Debug, Clone)]
pub struct Client<D>
where
    D: Database + Send + Sync + Sized + 'static,
{
    id: String,
    pub database: D,
}

impl<D> Client<D>
where
    D: Database,
{
    pub(crate) fn new<S>(id: S, database: D) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            database,
        }
    }

    pub fn origin(&self) -> Id<Origin> {
        Id::new(self.id.clone())
    }

    pub async fn get_origins(&self) -> RequestResult<Vec<WithId<Origin>>> {
        Ok(self.database.auto().origins().await?)
    }

    pub async fn get_origin_ids(&self) -> RequestResult<Vec<Id<Origin>>> {
        self.get_origins()
            .await?
            .into_iter()
            .map(|origin| origin.id)
            .collect::<Vec<_>>()
            .let_owned(|ids| Ok(ids))
    }

    pub async fn merge_with_defaults<T>(
        &self,
        values: Vec<WithOrigin<T>>,
    ) -> RequestResult<T>
    where
        T: Mergable + Serialize + Clone,
    {
        let default_origin_order = self
            .get_origins()
            .await?
            .into_iter()
            .map(|origin| origin.id)
            .collect::<Vec<_>>();
        merge_all_from(values, &default_origin_order)
            .ok_or(crate::RequestError::NotFound)
    }
}

impl<D> Client<D>
where
    D: Database,
{
    pub async fn get_agency_id_by_original_id(
        &self,
        original_id: String,
    ) -> RequestResult<Option<Id<Agency>>> {
        SubjectRepo::<Agency>::id_by_original_id(
            &mut self.database.auto(),
            Id::new(self.id.clone()),
            original_id,
        )
        .await?
        .let_owned(Ok)
    }

    pub async fn get_agencies(
        &self,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<Vec<WithId<Agency>>> {
        self.database
            .auto()
            .get_all()
            .await?
            .merge_all_from(&origins)
            .let_owned(|agencies| Ok(agencies))
    }

    pub async fn get_agency(
        &self,
        id: Id<Agency>,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<WithId<Agency>> {
        let result = self.database.auto().get(id).await?;
        result
            .merge_from(&origins)
            .ok_or(crate::RequestError::NotFound)
    }

    pub async fn push_agency(
        &self,
        agency: Agency,
        original_id: Option<String>,
    ) -> RequestResult<WithOrigin<WithId<Agency>>> {
        let mut tx = self.database.transaction().await?;
        let agencies_with_same_name = tx.agency_by_name(&agency.name).await?;
        // insert into database
        let result: Result<_, RequestError> =
            if let Some(entry) = agencies_with_same_name.first() {
                let id = entry.id.clone();
                tx.put(WithOrigin::new(
                    Id::new(self.id.clone()),
                    WithId::new(id, agency),
                ))
                .await
            } else {
                tx.insert(WithOrigin::new(Id::new(self.id.clone()), agency))
                    .await
            }
            .map_err(|why| why.into());
        let result = result?;
        // insert original id if given
        if let Some(original_id) = original_id {
            tx.put_original_id(
                result.origin.clone(),
                original_id,
                result.content.id.clone(),
            )
            .await?;
        }
        // commit changes
        tx.commit().await.map(|_| result).map_err(|why| why.into())
    }
}

impl<D> Client<D>
where
    D: Database,
{
    pub async fn get_line_id_by_original_id(
        &self,
        original_id: String,
    ) -> RequestResult<Option<Id<Line>>> {
        SubjectRepo::<Line>::id_by_original_id(
            &mut self.database.auto(),
            Id::new(self.id.clone()),
            original_id,
        )
        .await?
        .let_owned(Ok)
    }

    pub async fn get_lines(
        &self,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<Vec<WithId<Line>>> {
        self.database
            .auto()
            .get_all()
            .await?
            .merge_all_from(&origins)
            .let_owned(Ok)
    }

    pub async fn get_line(
        &self,
        id: Id<Line>,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<WithId<Line>> {
        let result = self.database.auto().get(id).await?;
        result
            .merge_from(&origins)
            .ok_or(crate::RequestError::NotFound)
    }

    pub async fn push_line(
        &self,
        line: Line,
        original_id: Option<String>,
    ) -> RequestResult<WithOrigin<WithId<Line>>> {
        // TODO: lines with the same name and agency are currently merged.
        // This causes e.g, all db intercities to count as one line.
        let mut tx = self.database.transaction().await?;
        let lines_with_same_name = match (&line.name, &line.agency_id) {
            (Some(name), Some(agency)) => {
                Some(tx.line_by_name_and_agency(name, agency).await?)
            }
            _ => None,
        };
        // insert into database
        let result: Result<_, RequestError> = if let Some(entry) =
            lines_with_same_name.and_then(|vec| vec.first().cloned())
        {
            let id = entry.id.clone();
            tx.put(WithOrigin::new(
                Id::new(self.id.clone()),
                WithId::new(id, line),
            ))
            .await
        } else {
            tx.insert(WithOrigin::new(Id::new(self.id.clone()), line))
                .await
        }
        .map_err(|why| why.into());
        let result = result?;
        // insert orignal id if given
        if let Some(original_id) = original_id {
            tx.put_original_id(
                result.origin.clone(),
                original_id,
                result.content.id.clone(),
            )
            .await?;
        }
        // commit changes
        tx.commit().await.map(|_| result).map_err(|why| why.into())
    }

    pub async fn get_lines_at_stop(
        &self,
        stop_id: &Id<Stop>,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<WithId<Line>>> {
        self.database
            .auto()
            .get_by_stop_id(stop_id)
            .await?
            .merge_all_from(origins)
            .let_owned(Ok)
    }
}

impl<D> Client<D>
where
    D: Database,
{
    pub async fn get_stop_id_by_original_id(
        &self,
        original_id: String,
    ) -> RequestResult<Option<Id<Stop>>> {
        SubjectRepo::<Stop>::id_by_original_id(
            &mut self.database.auto(),
            Id::new(self.id.clone()),
            original_id,
        )
        .await?
        .let_owned(Ok)
    }

    pub async fn get_stops(
        &self,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<Vec<WithId<Stop>>> {
        self.database
            .auto()
            .get_all()
            .await?
            .merge_all_from(&origins)
            .let_owned(|stops| Ok(stops))
    }

    pub async fn get_stop(
        &self,
        id: Id<Stop>,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<WithId<Stop>> {
        let result = self.database.auto().get(id).await?;
        result
            .merge_from(&origins)
            .ok_or(crate::RequestError::NotFound)
    }

    pub async fn push_stop(
        &self,
        stop: Stop,
        original_id: Option<String>,
    ) -> RequestResult<WithOrigin<WithId<Stop>>> {
        let mut tx = self.database.transaction().await?;
        let origin = Id::new(self.id.clone());
        let stop_with_same_original_id = match &original_id {
            Some(original_id) => {
                self.get_stop_id_by_original_id(original_id.clone()).await?
            }
            None => None,
        };
        // insert into database
        let result: Result<_, RequestError> = if let Some(id) =
            stop_with_same_original_id
        {
            // insert with known mapping
            tx.put(WithOrigin::new(
                Id::new(self.id.clone()),
                WithId::new(id, stop),
            ))
            .await
        } else if let Some((similarity, same_subject)) =
            filter_sort_subjects(&stop, tx.merge_candidates(&stop, &origin).await?)
                .first()
        {
            println!(
                "Identified Stops {}::'{}' and {}::'{}' to be Subject-Equal. Confidence: {}.",
                origin,
                stop.name.as_ref().unwrap_or(&"<unknown>".to_owned()),
                same_subject.origin.raw_ref::<str>(),
                same_subject
                    .content
                    .content
                    .name
                    .as_ref()
                    .unwrap_or(&"<unknown>".to_owned()),
                similarity
            );
            // insert with identified subject
            tx.put(WithOrigin::new(
                origin.clone(),
                WithId::new(same_subject.content.id.clone(), stop),
            ))
            .await
        } else {
            // insert completely new
            tx.insert(WithOrigin::new(Id::new(self.id.clone()), stop))
                .await
        }
        .map_err(|why| why.into());
        let result = result?;
        // insert original id if given
        if let Some(original_id) = original_id {
            tx.put_original_id(
                result.origin.clone(),
                original_id,
                result.content.id.clone(),
            )
            .await?;
        }
        // commit changes
        tx.commit().await.map(|_| result).map_err(|why| why.into())
    }

    pub async fn find_nearby(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<WithDistance<WithId<Stop>>>> {
        self.database
            .auto()
            .find_nearby(latitude, longitude, radius_km)
            .await?
            .merge_all_from(origins)
            .into_iter()
            .filter_map(|stop| {
                stop.content
                    .with_distance_to(latitude, longitude)
                    .map(|with_distance| with_distance.with_id(stop.id))
            })
            .collect::<Vec<_>>()
            .let_owned(|stops| Ok(stops))
    }

    pub async fn search_stop<S: Into<String>>(
        &self,
        pattern: S,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<StopNameSuggestion>> {
        self.database
            .auto()
            .search(pattern.into())
            .await?
            .merge_all_from(origins)
            .into_iter()
            .filter_map(|stop| match (stop.content.name, stop.content.location) {
                (Some(name), Some(location)) => Some(StopNameSuggestion {
                    id: stop.id,
                    name,
                    latitude: location.latitude,
                    longitude: location.longitude,
                }),
                _ => None,
            })
            .collect::<Vec<_>>()
            .let_owned(|stops| Ok(stops))
    }
}

impl<D> Client<D>
where
    D: Database,
{
    pub async fn get_trip_id_by_original_id(
        &self,
        original_id: String,
    ) -> RequestResult<Option<Id<Trip>>> {
        SubjectRepo::<Trip>::id_by_original_id(
            &mut self.database.auto(),
            Id::new(self.id.clone()),
            original_id,
        )
        .await?
        .let_owned(Ok)
    }

    pub async fn get_trips(
        &self,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<Vec<WithId<Trip>>> {
        // todo: insert stops
        self.database
            .auto()
            .get_all()
            .await?
            .merge_all_from(&origins)
            .let_owned(|trips| Ok(trips))
    }

    pub async fn get_trip(
        &self,
        id: Id<Trip>,
        origins: Vec<Id<Origin>>,
    ) -> RequestResult<WithId<Trip>> {
        let mut result = self.database.auto().get(id.clone()).await?;
        self.with_stop_times(&mut result).await?;
        result
            .merge_from(&origins)
            .ok_or(crate::RequestError::NotFound)
    }

    pub async fn push_trip(
        &self,
        mut trip: Trip,
        original_id: Option<String>,
        clear_stop_times: bool,
    ) -> RequestResult<WithOrigin<WithId<Trip>>> {
        // TODO: think about how to identify trips from different sources as the same.
        let mut tx = self.database.transaction().await?;
        let stop_times = trip.stops.drain(..).collect::<Vec<_>>();
        let origin = Id::new(self.id.clone());
        let trip_with_same_original_id = match &original_id {
            Some(original_id) => {
                self.get_trip_id_by_original_id(original_id.clone()).await?
            }
            None => None,
        };
        // insert into database
        let result: Result<_, RequestError> =
            if let Some(id) = trip_with_same_original_id {
                tx.put(WithOrigin::new(origin.clone(), WithId::new(id, trip)))
                    .await
            } else {
                tx.insert(WithOrigin::new(Id::new(self.id.clone()), trip))
                    .await
            }
            .map_err(|why| why.into());
        let result = result?;
        // delete stop times (if existant from older version)
        if clear_stop_times {
            tx.delete_stop_times(result.content.id.clone(), Id::new(self.id.clone()))
                .await?;
        }
        // insert stops (if given)
        for stop_time in stop_times {
            tx.put_stop_time(
                result.content.id.clone(),
                WithOrigin::new(result.origin.clone(), stop_time),
            )
            .await?;
        }
        // insert original id if given
        if let Some(original_id) = original_id {
            tx.put_original_id(
                result.origin.clone(),
                original_id,
                result.content.id.clone(),
            )
            .await?;
        }
        // commit changes
        tx.commit().await.map(|_| result).map_err(|why| why.into())
    }

    pub async fn push_stop_time(
        &self,
        trip_id: Id<Trip>,
        stop_time: StopTime,
    ) -> RequestResult<WithOrigin<StopTime>> {
        self.database
            .auto()
            .put_stop_time(
                trip_id,
                WithOrigin::new(Id::new(self.id.clone()), stop_time),
            )
            .await?
            .let_owned(Ok)
    }

    pub async fn get_all_trips_via_stops(
        &self,
        stop_ids: &[&Id<Stop>],
        start: DateTime<Local>,
        end: DateTime<Local>,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<WithId<Trip>>> {
        let mut result = self
            .database
            .auto()
            .get_all_via_stop(
                stop_ids,
                // Since trips that extend beyond one day have arrival and departure
                // times past midnight and still belong to the previous day, the
                // previous day must always be included in a request.
                start - Duration::days(1),
                end,
            )
            .await?;

        for entry in result.iter_mut() {
            self.with_stop_times(entry).await?;
        }

        Ok(result.merge_all_from(&origins))
    }

    /// Instancates the given trips in the given range and includes other information.
    ///
    /// # WARNING
    ///
    /// Currently, lines are always included if you include agencies.
    /// TODO: introduce repo function to fetch agency by `line_id` to solve this
    ///       issue.
    pub async fn instanciate_trips_include(
        &self,
        trips: Vec<WithId<Trip>>,
        range: DateTimeRange<Local>,
        stop_ids_of_interest: Option<&[&Id<Stop>]>,
        include_stop_names: bool,
        include_lines: bool,
        include_agencies: bool,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<TripInstance>> {
        let mut trips = self
            .instanciate_trips(trips, range, stop_ids_of_interest)
            .await?;

        let mut stops: HashMap<Id<Stop>, Option<Stop>> = HashMap::new();
        let mut lines: HashMap<Id<Line>, Option<WithId<Line>>> = HashMap::new();
        let mut agencies: HashMap<Id<Agency>, Option<WithId<Agency>>> =
            HashMap::new();

        for trip in trips.iter_mut() {
            // lines
            if include_lines || include_agencies {
                let id = &trip.info.line_id;
                trip.line = if let Some(cached) = lines.get(id) {
                    cached.clone()
                } else {
                    let fetched = self
                        .get_line(id.clone(), origins.to_vec())
                        .await
                        .let_owned(not_found_to_none)?;
                    lines.insert(id.clone(), fetched.clone());
                    fetched
                };
            }
            // agencies
            if include_agencies {
                if let Some(id) = trip
                    .line
                    .as_ref()
                    .and_then(|line| line.content.agency_id.as_ref())
                {
                    trip.agency = if let Some(cached) = agencies.get(id) {
                        cached.clone()
                    } else {
                        let fetched = self
                            .get_agency(id.clone(), origins.to_vec())
                            .await
                            .let_owned(not_found_to_none)?;
                        agencies.insert(id.clone(), fetched.clone());
                        fetched
                    };
                }
            }
            // stop names
            if include_stop_names {
                for stop_time in trip
                    .stops
                    .iter_mut()
                    .chain(trip.stop_of_interest.iter_mut())
                {
                    if let Some(id) = stop_time.stop_id.as_ref() {
                        let stop = if let Some(cached) = stops.get(id) {
                            cached.clone()
                        } else {
                            let fetched = self
                                .get_stop(id.clone(), origins.to_vec())
                                .await
                                .let_owned(not_found_to_none)?
                                .map(|stop| stop.content);
                            stops.insert(id.clone(), fetched.clone());
                            fetched
                        };
                        if let Some(stop) = stop {
                            stop_time.stop_name = stop.name;
                            stop_time.location = stop.location;
                        }
                    }
                }

                // if no headsign -> set to last stop with name
                trip.info.headsign = trip
                    .info
                    .headsign
                    .as_ref()
                    .or_else(|| {
                        trip.stops
                            .iter()
                            .rev()
                            .filter_map(|s| s.stop_name.as_ref())
                            .next()
                    })
                    .cloned();
            }
        }

        Ok(trips)
    }

    /// Instanciates the passed trips within a given datetime range at the given
    /// stop ids. Each trip is only instaciated once, even if it stops at more than
    /// one of the provided stop ids. In the latter case, stop ids are prioritized
    /// by position in the array.
    pub async fn instanciate_trips(
        &self,
        trips: Vec<WithId<Trip>>,
        range: DateTimeRange<Local>,
        stop_ids_of_interest: Option<&[&Id<Stop>]>, // accept multiple ids an prioritize by position in array.
    ) -> RequestResult<Vec<TripInstance>> {
        let start: DateTime<Local> = range.first;
        let end: DateTime<Local> = range.last;

        let mut days_of_services: HashMap<Id<Service>, Vec<NaiveDate>> =
            HashMap::new();

        let mut results = vec![];

        // instanciate trips
        for trip in trips {
            // trips without service_id can not be instanciated.
            let service_id = if let Some(id) = trip.content.service_id {
                id
            } else {
                continue;
            };
            // get available days of service within date span of interest.
            let days = if let Some(cached) = days_of_services.get(&service_id) {
                cached.clone()
            } else {
                let available = self.get_service(&service_id).await?.available_days(
                    Some(start.date_naive() - Duration::days(1)),
                    Some(end.date_naive()),
                );
                days_of_services.insert(service_id, available.clone());
                available
            };
            // instanciate trip for each service day within interest window.
            let result = days.iter().filter_map(|day| {
                instantiate_trip_naive(&trip, day, Some(&range), stop_ids_of_interest)
            });
            results.extend(result);
        }

        Ok(results)
    }

    async fn with_stop_times(
        &self,
        entry: &mut DatabaseEntry<Trip>,
    ) -> RequestResult<()> {
        for source in entry.source_data.iter_mut() {
            let mut stops = self
                .database
                .auto()
                .get_stop_times(entry.id.clone(), source.origin.clone())
                .await?;
            // muss das? oder sortier ich schon wo anders? ich weiß es nicht.
            stops.sort_by_key(|stop| stop.stop_sequence);
            source.content.stops = stops;
        }
        Ok(())
    }
}

/// Instantiates the trip for the given date, regardless of the trip is serviced
/// on that that particular date (thus naive).
/// If `range` or `stop_ids_of_interest` are given, the trip is only instantiated,
/// if these filters match.
/// If these are not specified, the trip is always instantiated.
pub fn instantiate_trip_naive(
    trip: &WithId<Trip>,
    date: &NaiveDate,
    range: Option<&DateTimeRange<Local>>,
    stop_ids_of_interest: Option<&[&Id<Stop>]>,
) -> Option<TripInstance> {
    // common trip instance info.
    let trip_info = TripInstanceInfo {
        trip_id: trip.id.clone(),
        line_id: trip.content.line_id.clone(),
        service_id: trip.content.service_id,
        headsign: trip.content.headsign.clone(),
        short_name: trip.content.short_name.clone(),
    };
    // local datetime
    let datetime = date
        .and_time(NaiveTime::default())
        .and_local_timezone(Local)
        .earliest()?; // TODO: handle invalid date
    let mut stop_time_instance_of_interest_idx = None; // index of stop of interst in stop_ids
    let mut stop_time_instance_of_interest = None;
    let mut instance_headsign = trip_info.headsign.clone();
    let stop_times = trip
        .content
        .stops
        .iter()
        .map(|stop_time| {
            // calculate arrival and departure time.
            let arrival_time = stop_time.arrival_time.map(|time| datetime + time);
            let departure_time = stop_time.departure_time.map(|time| datetime + time);

            // if no headsign and this stop comes before or at stop of interest...
            if let (None, Some(stop_headsign)) =
                (&stop_time_instance_of_interest, &stop_time.stop_headsign)
            {
                instance_headsign = Some(stop_headsign.clone());
            }

            // is stop of interest?
            let idx = stop_time.stop_id.as_ref().and_then(|stop_id| {
                stop_ids_of_interest
                    .and_then(|ids| ids.iter().position(|i| i == &stop_id))
            });
            let is_stop_of_interest = idx.is_some() || stop_ids_of_interest.is_none();

            // is time in frame?
            let is_time_of_interest = if let Some(range) = range.as_ref() {
                let start = range.first;
                let end = range.last;
                arrival_time
                    .map(|time| time >= start && time <= end)
                    .unwrap_or(false)
                    || departure_time
                        .map(|time| time >= start && time <= end)
                        .unwrap_or(false)
            } else {
                true
            };

            // is stop_time combined interesting?
            let is_stop_time_of_interest = is_stop_of_interest && is_time_of_interest;

            let stop_time_instance = StopTimeInstance {
                stop_sequence: stop_time.stop_sequence,
                stop_id: stop_time.stop_id.clone(),
                stop_name: None,
                arrival_time,
                departure_time,
                stop_headsign: stop_time.stop_headsign.clone(),
                interest_flag: is_stop_time_of_interest,
                location: None,
            };

            // update stop time of interest.
            // TODO: unnötiges rumgeklone kann man verhindern.
            if is_stop_time_of_interest {
                match (idx, stop_time_instance_of_interest_idx) {
                    // is stop time of interest and has higher prio than current?
                    (Some(this), Some(curr)) => {
                        if this < curr {
                            stop_time_instance_of_interest =
                                Some(stop_time_instance.clone());
                            stop_time_instance_of_interest_idx = idx;
                        }
                    }
                    // is first found stop timie of interest?
                    (Some(_), None) => {
                        stop_time_instance_of_interest =
                            Some(stop_time_instance.clone());
                        stop_time_instance_of_interest_idx = idx;
                    }
                    // stop time is not interesting
                    _ => {}
                }
            }

            stop_time_instance
        })
        .collect::<Vec<_>>();

    stop_time_instance_of_interest.map(|stop_of_interest| TripInstance {
        info: trip_info
            .clone()
            .let_owned(|mut trip_info: TripInstanceInfo| {
                trip_info.headsign = instance_headsign;
                trip_info
            }),
        stops: stop_times,
        stop_of_interest: if stop_ids_of_interest.is_some() || range.is_some() {
            Some(stop_of_interest)
        } else {
            None
        },
        line: None,
        agency: None,
    })
}

impl<D> Client<D>
where
    D: Database,
{
    pub async fn get_service_id_by_original_id(
        &self,
        original_id: String,
    ) -> RequestResult<Option<Id<Service>>> {
        SubjectRepo::<Service>::id_by_original_id(
            &mut self.database.auto(),
            Id::new(self.id.clone()),
            original_id,
        )
        .await?
        .let_owned(Ok)
    }

    pub async fn get_service(
        &self,
        service_id: &Id<Service>,
    ) -> RequestResult<Service> {
        let windows = self
            .database
            .auto()
            .get_calendar_windows(service_id)
            .await?;
        let dates = self.database.auto().get_calendar_dates(service_id).await?;
        Ok(Service { windows, dates })
    }

    pub async fn push_calendar_window<S>(
        &self,
        service_id: Option<&Id<Service>>,
        window: CalendarWindow,
        original_id: Option<S>,
    ) -> RequestResult<(Id<Service>, CalendarWindow)>
    where
        S: Into<String>,
    {
        if let (Some(original_id), None) = (original_id, service_id) {
            let mut tx = self.database.transaction().await?;
            let (id, result) = tx.put_calendar_window(service_id, window).await?;
            SubjectRepo::put_original_id(
                &mut tx,
                Id::new(self.id.clone()),
                original_id.into(),
                id,
            )
            .await?;
            tx.commit().await?;
            Ok((id, result))
        } else {
            self.database
                .auto()
                .put_calendar_window(service_id, window)
                .await
                .map_err(From::from)
        }
    }

    pub async fn push_calendar_date<S>(
        &self,
        service_id: Option<&Id<Service>>,
        date: CalendarDate,
        original_id: Option<S>,
    ) -> RequestResult<(Id<Service>, CalendarDate)>
    where
        S: Into<String>,
    {
        if let (Some(original_id), None) = (original_id, service_id) {
            let mut tx = self.database.transaction().await?;
            let (id, result) = tx.put_calendar_date(service_id, date).await?;
            SubjectRepo::put_original_id(
                &mut tx,
                Id::new(self.id.clone()),
                original_id.into(),
                id,
            )
            .await?;
            tx.commit().await?;
            Ok((id, result))
        } else {
            self.database
                .auto()
                .put_calendar_date(service_id, date)
                .await
                .map_err(From::from)
        }
    }
}

/// realtime data
impl<D> Client<D>
where
    D: Database,
{
    pub async fn put_trip_updates(
        &self,
        updates: Vec<WithId<TripUpdate>>,
    ) -> RequestResult<Vec<WithId<TripUpdate>>> {
        let origin = Id::new(self.id.clone());
        let mut tx = self.database.transaction().await?;
        let mut new_updates = vec![];
        for update in updates {
            let update_id = update.id.raw();
            let timestamp = tx
                .get_timestamp(&origin, &update_id.trip_id, update_id.trip_start_date)
                .await?;
            let is_new = update
                .content
                .timestamp
                .and_then(|new_ts| timestamp.map(|old_ts| new_ts > old_ts))
                .unwrap_or(true);
            if is_new {
                new_updates.push(update);
            }
        }
        for chunk in new_updates.chunks(D::BULK_INSERT_MAX) {
            tx.put_trip_updates(&Id::new(self.id.clone()), chunk)
                .await?
                .content;
        }
        tx.commit().await?;
        Ok(new_updates)
    }

    pub async fn put_stop_time_update(
        &self,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
        stop_time: StopTimeUpdate,
    ) -> RequestResult<()> {
        let mut tx = self.database.transaction().await?;
        let realtime = if let Some(mut current) = tx
            .get_realtime_for_trip(trip_id, trip_start_date)
            .await?
            .merge_from(&[Id::new(self.id.clone())])
        {
            let mut set = false;
            for stop_update in current.content.stops.iter_mut() {
                let is_same = stop_update
                    .scheduled_stop_sequence
                    .as_ref()
                    .and_then(|u| {
                        stop_time.scheduled_stop_sequence.as_ref().map(|t| u == t)
                    })
                    .unwrap_or(false);
                if is_same {
                    *stop_update = stop_time.clone();
                    set = true;
                    break;
                }
            }
            if !set {
                current.content.stops.push(stop_time);
            }
            current
        } else {
            WithId::new(
                Id::new(TripUpdateId::new(trip_id.clone(), trip_start_date)),
                TripUpdate {
                    status: TripStatus::Scheduled,
                    stops: vec![stop_time],
                    timestamp: Some(Local::now()),
                },
            )
        };
        tx.put_trip_updates(&Id::new(self.id.clone()), &[realtime])
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_realtime_for_trip(
        &self,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
        origins: &[Id<Origin>],
    ) -> RequestResult<WithId<TripUpdate>> {
        self.database
            .auto()
            .get_realtime_for_trip(trip_id, trip_start_date)
            .await?
            .merge_from(origins)
            .ok_or(crate::RequestError::NotFound)
    }

    pub async fn get_realtime_for_trips_in_range<'c>(
        &self,
        trip_ids: &[Id<Trip>],
        range: DateTimeRange<Local>,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<WithId<TripUpdate>>> {
        self.database
            .auto()
            .get_realtime_for_trips_in_range(trip_ids, range)
            .await?
            .merge_all_from(origins)
            .let_owned(Ok)
    }
}

/// shared mobility
impl<D> Client<D>
where
    D: Database,
{
    pub async fn put_shared_mobility_stations(
        &self,
        stations: Vec<WithId<SharedMobilityStation>>,
    ) -> RequestResult<Vec<WithId<SharedMobilityStation>>> {
        let origin = Id::new(self.id.clone());
        let mut tx = self.database.transaction().await?;
        for chunk in stations.chunks(D::BULK_INSERT_MAX) {
            tx.put_shared_mobility_stations(&origin, chunk)
                .await?
                .content;
        }
        tx.commit().await?;
        // TODO: insert id mappings
        Ok(stations)
    }

    pub async fn update_shared_mobility_station_status(
        &self,
        id: &Id<SharedMobilityStation>,
        status: Option<Status>,
    ) -> RequestResult<()> {
        self.database
            .auto()
            .update_shared_mobility_station_status(
                &Id::new(self.id.clone()),
                id,
                status,
            )
            .await?;
        Ok(())
    }

    pub async fn find_nearby_shared_mobility_stations(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
        origins: &[Id<Origin>],
    ) -> RequestResult<Vec<WithDistance<WithId<SharedMobilityStation>>>> {
        self.database
            .auto()
            .find_nearby_shared_mobility_stations(latitude, longitude, radius_km)
            .await?
            .merge_all_from(origins)
            .into_iter()
            .filter_map(|stop| {
                stop.content
                    .with_distance_to(latitude, longitude)
                    .map(|with_distance| with_distance.with_id(stop.id))
            })
            .collect::<Vec<_>>()
            .let_owned(|stops| Ok(stops))
    }
}
