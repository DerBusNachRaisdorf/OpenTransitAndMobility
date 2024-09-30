use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;

use serde::de::DeserializeOwned;

use crate::data_model::agency::{Agency, AgencyId};
use crate::data_model::routes::{Route, RouteId};
use crate::data_model::stop_times::{StopTime, StopTimeKey};
use crate::data_model::stops::{Stop, StopId};
use crate::data_model::trips::{Trip, TripId};

pub trait WithPrimaryKey<K> {
    fn primary_key(&self) -> K;
}

pub trait PrimaryKeyTable<K, V>
where
    V: WithPrimaryKey<K>,
{
    fn get(&self, key: &K) -> Option<V>;

    fn get_all(&self) -> Vec<V>;

    fn insert(&mut self, value: V);
}

pub fn read_csv<T, K, V, R>(table: &mut T, reader: R) -> Result<(), Box<dyn Error>>
where
    T: PrimaryKeyTable<K, V>,
    V: WithPrimaryKey<K>,
    V: DeserializeOwned,
    R: Read,
{
    let mut csv_reader = csv::Reader::from_reader(reader);
    for row in csv_reader.deserialize() {
        let value: V = row?;
        table.insert(value);
    }
    Ok(())
}

pub fn read_csv_file<T, K, V, R>(table: &mut T, file_path: &str) -> Result<(), Box<dyn Error>>
where
    T: PrimaryKeyTable<K, V>,
    V: WithPrimaryKey<K>,
    V: DeserializeOwned,
    R: Read,
{
    read_csv(table, File::open(file_path)?)
}

pub struct InMemoryPrimaryKeyTable<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> InMemoryPrimaryKeyTable<K, V>
where
    K: Eq,
    K: Hash,
    V: WithPrimaryKey<K>,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn from_rows(rows: Vec<V>) -> Self {
        let mut map = HashMap::<K, V>::new();
        for row in rows {
            map.insert(row.primary_key(), row);
        }
        Self { map }
    }
}

impl<K, V> PrimaryKeyTable<K, V> for InMemoryPrimaryKeyTable<K, V>
where
    K: Eq,
    K: Hash,
    K: Debug,
    V: Clone,
    V: WithPrimaryKey<K>,
{
    fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).cloned()
    }

    fn get_all(&self) -> Vec<V> {
        self.map.values().cloned().collect::<Vec<_>>()
    }

    fn insert(&mut self, value: V) {
        println!("inserted with key: {:?}", value.primary_key());
        self.map.insert(value.primary_key(), value);
    }
}

pub type InMemoryAgencyTable = InMemoryPrimaryKeyTable<Option<AgencyId>, Agency>;
pub type InMemoryStopTable = InMemoryPrimaryKeyTable<StopId, Stop>;
pub type InMemoryRouteTable = InMemoryPrimaryKeyTable<RouteId, Stop>;
pub type InMemoryTripTable = InMemoryPrimaryKeyTable<TripId, Stop>;
pub type InMemoryStopTimeTable = InMemoryPrimaryKeyTable<StopTimeKey, Stop>;

pub type AgencyTable = dyn PrimaryKeyTable<Option<AgencyId>, Agency>;
pub type StopTable = dyn PrimaryKeyTable<StopId, Stop>;
pub type RouteTable = dyn PrimaryKeyTable<RouteId, Route>;
pub type TripTable = dyn PrimaryKeyTable<TripId, Trip>;
pub type StopTimeTable = dyn PrimaryKeyTable<StopTimeKey, StopTime>;

pub struct GtfsDatabase {
    /// Transit agencies with service represented in this dataset.
    pub agency: Box<AgencyTable>,

    /// Stops where vehicles pick up or drop off riders.
    /// Also defines stations and station entrances.
    pub stops: Box<StopTable>,

    /// Transit routes.
    /// A route is a group of trips that are displayed to riders as a single service.
    pub routes: Box<RouteTable>,

    /// Trips for each route. A trip is a sequence of two or more stops that occur
    /// during a specific time period.
    pub trips: Box<TripTable>,

    /// Times that a vehicle arrives at and departs from stops for each trip.
    pub stop_times: Box<StopTimeTable>,
}

impl GtfsDatabase {
    pub fn new_in_memory() -> Self {
        GtfsDatabase {
            agency: Box::new(InMemoryPrimaryKeyTable::new()),
            stops: Box::new(InMemoryPrimaryKeyTable::new()),
            routes: Box::new(InMemoryPrimaryKeyTable::new()),
            trips: Box::new(InMemoryPrimaryKeyTable::new()),
            stop_times: Box::new(InMemoryPrimaryKeyTable::new()),
        }
    }
}
