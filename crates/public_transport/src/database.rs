use std::{error, fmt::Debug, future::Future, result};

use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate};
use model::{
    agency::Agency,
    calendar::{CalendarDate, CalendarWindow, Service},
    line::Line,
    origin::{Origin, OriginalIdMapping},
    shared_mobility::{SharedMobilityStation, Status},
    stop::Stop,
    trip::{StopTime, Trip},
    trip_update::TripUpdate,
    DatabaseEntry, DateTimeRange, WithId, WithOrigin,
};
use serde::Serialize;
use utility::id::{HasId, Id};

use crate::collector::{Collector, CollectorInstance};

#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    IdMissing,
    Other(Box<dyn error::Error + Send + Sync>),
}

pub type Result<T> = result::Result<T, DatabaseError>;

#[async_trait]
pub trait Repo<T: Serialize + HasId>
where
    <T as HasId>::IdType: Debug + Clone + Serialize,
{
    async fn get(&mut self, id: Id<T>) -> Result<DatabaseEntry<T>>;
    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<T>>>;
    async fn insert(
        &mut self,
        element: WithOrigin<T>,
    ) -> Result<WithOrigin<WithId<T>>>;
    async fn put(
        &mut self,
        element: WithOrigin<WithId<T>>,
    ) -> Result<WithOrigin<WithId<T>>>;
    async fn update(
        &mut self,
        element: WithOrigin<WithId<T>>,
    ) -> Result<WithOrigin<WithId<T>>>;
    async fn exists(&mut self, id: Id<T>) -> Result<bool>;
    async fn exists_with_origin(
        &mut self,
        id: Id<T>,
        origin: Id<Origin>,
    ) -> Result<bool>;
}

/// A repo which is the main repo for a subject.
#[async_trait]
pub trait SubjectRepo<S>
where
    S: Serialize + HasId,
    <S as HasId>::IdType: Debug + Clone + Serialize,
{
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<S>>>;

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<S>,
    ) -> Result<OriginalIdMapping<S>>;
}

#[async_trait]
pub trait MergableRepo<S>
where
    S: Serialize + HasId,
    <S as HasId>::IdType: Debug + Clone + Serialize,
{
    /// Searches for similar existing entries in the database, which might then
    /// be used to identify an element to merge the passed element with.
    async fn merge_candidates(
        &mut self,
        element: &S,
        excluded_origin: &Id<Origin>,
    ) -> Result<Vec<WithOrigin<WithId<S>>>>;
}

#[async_trait]
pub trait AgencyRepo: SubjectRepo<Agency> + Repo<Agency> {
    async fn agency_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Agency>>>;
}

#[async_trait]
pub trait LineRepo: SubjectRepo<Line> + Repo<Line> {
    async fn line_by_name_and_agency<S: Into<String> + Send>(
        &mut self,
        name: S,
        agency: &Id<Agency>,
    ) -> Result<Vec<DatabaseEntry<Line>>>;

    async fn get_by_stop_id(
        &mut self,
        stop_id: &Id<Stop>,
    ) -> Result<Vec<DatabaseEntry<Line>>>;
}

#[async_trait]
pub trait StopRepo: SubjectRepo<Stop> + Repo<Stop> + MergableRepo<Stop> {
    async fn find_nearby(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<Stop>>>;

    async fn stop_by_name<S: Into<String> + Send>(
        &mut self,
        name: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>>;

    async fn search<S: Into<String> + Send>(
        &mut self,
        pattern: S,
    ) -> Result<Vec<DatabaseEntry<Stop>>>;
}

#[async_trait]
pub trait TripRepo: SubjectRepo<Trip> + Repo<Trip> {
    async fn put_stop_time(
        &mut self,
        trip_id: Id<Trip>,
        stop_time: WithOrigin<StopTime>,
    ) -> Result<WithOrigin<StopTime>>;

    async fn get_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<Vec<StopTime>>;

    // TODO: return deleted data
    async fn delete_stop_times(
        &mut self,
        trip_id: Id<Trip>,
        origin: Id<Origin>,
    ) -> Result<()>;

    /// Returns all trips, which stop at the specified stop.
    ///
    /// TODO: maybe take a naive date rather than a datetime, as checking a date and
    ///       time range requires instanciation of the trips, which a database can
    ///       not do.
    ///
    /// TODO: take separate time range to filter trips based on time of day.
    ///
    /// TODO: take optional list of stops where the trip should also stop at.
    ///       maybe make that a separate method. This could be used to implement
    ///       routing later.
    ///
    /// # WARNING
    ///
    /// This filters the data as much as possible at database level, but
    /// implementations are required to return too many trips rather than omitting
    /// some. Thus, the data returned has to be further filtered to guarantee
    /// that all trips actually stop at the given stop within the given date range.
    async fn get_all_via_stop(
        &mut self,
        stops: &[&Id<Stop>],
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<DatabaseEntry<Trip>>>;
}

#[async_trait]
pub trait ServiceRepo: SubjectRepo<Service> {
    /// inserts or updates a single calendar window into the database.
    /// updates the existing window if `service_id`, `start_date` and `end_date` are
    /// equal, inserts otherwise.
    ///
    /// creates a new `service_id` if none specified
    async fn put_calendar_window(
        &mut self,
        service_id: Option<&Id<Service>>,
        window: CalendarWindow,
    ) -> Result<(Id<Service>, CalendarWindow)>;

    /// inserts or updates a single calendar date into the database.
    /// updates the existing date if `service_id`, `date` are equal, inserts otherwise.
    ///
    /// creates a new `service_id` if none specified
    async fn put_calendar_date(
        &mut self,
        service_id: Option<&Id<Service>>,
        date: CalendarDate,
    ) -> Result<(Id<Service>, CalendarDate)>;

    /// obtains all calendar windows associated with a service.
    async fn get_calendar_windows(
        &mut self,
        service_id: &Id<Service>,
    ) -> Result<Vec<CalendarWindow>>;

    /// obtains all calendar dates associated with a service.
    async fn get_calendar_dates(
        &mut self,
        service_id: &Id<Service>,
    ) -> Result<Vec<CalendarDate>>;
}

#[async_trait]
pub trait RealtimeRepo {
    /// push updates for mutliple trips.
    ///
    /// ## Warning
    ///
    /// Push at most `Database::BULK_INSERT_MAX` number of updates at once.
    async fn put_trip_updates(
        &mut self,
        origin: &Id<Origin>,
        updates: &[WithId<TripUpdate>],
    ) -> Result<WithOrigin<Vec<WithId<TripUpdate>>>>;

    /// get realtime info for a specific trip on a specific day.
    async fn get_realtime_for_trip(
        &mut self,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<DatabaseEntry<TripUpdate>>;

    /// return the update timestamp (if set) of the realtime info (if exists) for the
    /// specified trip instance.
    async fn get_timestamp(
        &mut self,
        origin: &Id<Origin>,
        trip_id: &Id<Trip>,
        trip_start_date: NaiveDate,
    ) -> Result<Option<DateTime<Local>>>;

    /// returns all updates for the specified trips in the specified date-time range.
    ///
    /// # WARNING
    ///
    /// Might, depending on the database implementation, return trip updates which do
    /// not meet the specified parameters for performance reasons. This are typically
    /// elements, where testing for the specified parameters is expensive and thus
    /// left to the user to decide when it is best to check for those parameters.
    async fn get_realtime_for_trips_in_range<'c>(
        &mut self,
        trip_id: &[Id<Trip>],
        range: DateTimeRange<Local>,
    ) -> Result<Vec<DatabaseEntry<TripUpdate>>>;
}

#[async_trait]
pub trait SharedMobilityStationRepo: SubjectRepo<SharedMobilityStation> {
    async fn find_nearby_shared_mobility_stations(
        &mut self,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> Result<Vec<DatabaseEntry<SharedMobilityStation>>>;

    async fn put_shared_mobility_stations(
        &mut self,
        origin: &Id<Origin>,
        stations: &[WithId<SharedMobilityStation>],
    ) -> Result<WithOrigin<Vec<WithId<SharedMobilityStation>>>>;

    async fn update_shared_mobility_station_status(
        &mut self,
        origin: &Id<Origin>,
        id: &Id<SharedMobilityStation>,
        status: Option<Status>,
    ) -> Result<()>;
}

#[async_trait]
pub trait CollectorRepo {
    async fn collectors<C>(&mut self) -> Result<Vec<WithId<CollectorInstance<C>>>>
    where
        C: Collector + 'static;

    async fn get_collector<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
    ) -> Result<CollectorInstance<C>>
    where
        C: Collector + 'static;

    async fn set_collector_state<C>(
        &mut self,
        id: &Id<CollectorInstance<C>>,
        state: C::State,
    ) -> Result<C::State>
    where
        C: Collector + 'static;
}

#[async_trait]
pub trait DatabaseOperations:
    AgencyRepo
    + LineRepo
    + StopRepo
    + TripRepo
    + ServiceRepo
    + RealtimeRepo
    + SharedMobilityStationRepo
    + CollectorRepo
{
    /// Returns all known origins sorted by their priority. Last element has highest priority.
    async fn origins(&mut self) -> Result<Vec<WithId<Origin>>>;

    async fn put_origin(&mut self, origin: WithId<Origin>) -> Result<WithId<Origin>>;
}

#[async_trait]
pub trait DatabaseTransaction: DatabaseOperations {
    async fn commit(self) -> Result<()>;
}

pub trait DatabaseAutocommit: DatabaseOperations {}

/// trait to implement a public transport database.
/// multiple concurrent accesses should be possible by e.g. cloning the database object.
#[async_trait]
pub trait Database: Clone + Send + Sync + Sized {
    type Transaction: DatabaseTransaction + Send;
    type Autocommit: DatabaseAutocommit + Send;

    const BULK_INSERT_MAX: usize;

    async fn transaction(&self) -> Result<Self::Transaction>;

    fn auto(&self) -> Self::Autocommit;

    // maybe deprecate
    async fn perform_transaction<T, F, Fut>(&self, action: F) -> Result<T>
    where
        T: Send,
        F: Send + FnOnce(&mut Self::Transaction) -> Fut + Send,
        Fut: Future<Output = Result<T>> + Send;
}
