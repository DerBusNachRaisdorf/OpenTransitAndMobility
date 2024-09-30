use chrono::{DateTime, TimeZone};
use indexmap::IndexMap;
use origin::Origin;
use schemars::JsonSchema;
use std::{fmt::Debug, hash::Hash};

use serde::{Deserialize, Serialize};
pub use serde_with;
use utility::id::{HasId, Id};

pub mod agency;
pub mod calendar;
pub mod line;
pub mod origin;
pub mod shape;
pub mod shared_mobility;
pub mod stop;
pub mod trip;
pub mod trip_instance;
pub mod trip_update;

pub trait ExampleData {
    fn example_data() -> Self;
}

pub struct DateTimeRange<Tz>
where
    Tz: TimeZone,
{
    pub first: DateTime<Tz>,
    pub last: DateTime<Tz>,
}

impl<Tz: TimeZone> DateTimeRange<Tz> {
    pub fn new(first: DateTime<Tz>, last: DateTime<Tz>) -> Self {
        Self { first, last }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithDistance<T> {
    pub distance_km: f64,
    #[serde(flatten)]
    pub content: T,
}

impl<T> WithDistance<T> {
    pub fn new(distance_km: f64, content: T) -> Self {
        Self {
            distance_km,
            content,
        }
    }

    pub fn with_id(self, id: Id<T>) -> WithDistance<WithId<T>>
    where
        T: HasId,
        T::IdType: Debug + Clone + Serialize,
    {
        WithDistance::new(self.distance_km, WithId::new(id, self.content))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseEntry<V>
where
    V: Serialize + HasId,
    V::IdType: Debug + Clone + Serialize,
{
    pub id: Id<V>,
    pub source_data: Vec<WithOrigin<V>>,
}

impl<V> DatabaseEntry<V>
where
    V: Serialize + Mergable + HasId,
    V::IdType: Serialize + Debug + Clone,
{
    pub fn gather(id: Id<V>, data: Vec<WithOrigin<V>>) -> Self {
        Self {
            id,
            source_data: data,
        }
    }

    pub fn gather_many(values: Vec<WithOrigin<WithId<V>>>) -> Vec<Self>
    where
        V::IdType: Hash + Eq,
    {
        // Index map was used over hash map to preserve the order of the values,
        // in which they where returned from the database. This important because
        // ordering often has do be done at database level.
        // A possible drawback of using database level ordering is, that the order
        // does not reflect the origin prority, but that is acceptable.
        let mut by_ids: IndexMap<Id<V>, Self> = IndexMap::new();
        for value in values {
            if let Some(entry) = by_ids.get_mut(&value.content.id) {
                entry
                    .source_data
                    .push(WithOrigin::new(value.origin, value.content.content));
            } else {
                by_ids.insert(
                    value.content.id.clone(),
                    Self::gather(
                        value.content.id,
                        vec![WithOrigin::new(value.origin, value.content.content)],
                    ),
                );
            }
        }
        by_ids.into_values().collect::<Vec<_>>()
    }

    pub fn contains_data(&self) -> bool {
        !self.source_data.is_empty()
    }

    /// merge all source data in a somewhat random order.
    /// prefer using `merge_from`.
    pub fn merge(self) -> Option<WithId<V>> {
        merge_all(self.source_data).map(|value| WithId::new(self.id, value))
    }

    /// merge source data from the specified origins in the order of the 'origins' vec.
    pub fn merge_from(self, origins: &[Id<Origin>]) -> Option<WithId<V>>
    where
        V: Clone,
    {
        merge_all_from(self.source_data, origins)
            .map(|value| WithId::new(self.id, value))
    }

    pub fn merge_all_from(data: Vec<Self>, origins: &[Id<Origin>]) -> Vec<WithId<V>>
    where
        V: Clone,
    {
        data.into_iter()
            .map(|entry| entry.merge_from(origins))
            .filter(|entry| entry.is_some())
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>()
    }
}

pub trait DatabaseEntryCollection<V>
where
    V: Serialize + Mergable + HasId + Clone,
    V::IdType: Serialize + Debug + Clone,
{
    fn merge_all_from(self, origins: &[Id<Origin>]) -> Vec<WithId<V>>;
}

impl<V> DatabaseEntryCollection<V> for Vec<DatabaseEntry<V>>
where
    V: Serialize + Mergable + HasId + Clone,
    V::IdType: Serialize + Debug + Clone,
{
    fn merge_all_from(self, origins: &[Id<Origin>]) -> Vec<WithId<V>> {
        DatabaseEntry::merge_all_from(self, origins)
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WithId<V>
where
    V: HasId,
    V::IdType: Serialize + Debug + Clone,
{
    pub id: Id<V>,
    #[serde(flatten)]
    pub content: V,
}

impl<V> WithId<V>
where
    V: HasId,
    V::IdType: Serialize + Debug + Clone,
{
    pub fn new(id: Id<V>, content: V) -> Self {
        Self { id, content }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithOrigin<T: Serialize> {
    pub origin: Id<Origin>,

    #[serde(flatten)]
    pub content: T,
}

impl<T: Serialize> WithOrigin<T> {
    pub fn new(origin: Id<Origin>, content: T) -> Self {
        Self { origin, content }
    }
}

pub trait Mergable {
    /// Merges an other value to this. The other value has a higher priority.
    fn merge(self, other: Self) -> Self;
}

impl<T> Mergable for Option<T>
where
    T: Mergable,
{
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Some(old), Some(new)) => Some(old.merge(new)),
            (old @ Some(_), _) => old,
            (_, new @ Some(_)) => new,
            _ => None,
        }
    }
}

pub trait Subject {
    fn same_subject_as(&self, other: &Self) -> Option<f64>;
}

impl<T> Subject for Option<T>
where
    T: Subject,
{
    fn same_subject_as(&self, other: &Self) -> Option<f64> {
        match (self, other) {
            (Some(old), Some(new)) => old.same_subject_as(new),
            _ => None,
        }
    }
}

pub fn filter_sort_subjects<S: Subject + HasId>(
    element: &S,
    candidates: Vec<WithOrigin<WithId<S>>>,
) -> Vec<(f64, WithOrigin<WithId<S>>)>
where
    S: Serialize,
    S::IdType: Debug + Clone + Serialize,
{
    let mut result = candidates
        .into_iter()
        .filter_map(|candidate| {
            element
                .same_subject_as(&candidate.content.content)
                .map(|similarity| (similarity, candidate))
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });
    result
}

/// Merges all values in the order they appear in the vec.
pub fn merge_all<T>(values: Vec<WithOrigin<T>>) -> Option<T>
where
    T: Mergable + Serialize,
{
    let mut result: Option<T> = None;
    for next in values {
        match result {
            Some(last) => result = Some(last.merge(next.content)),
            None => result = Some(next.content),
        }
    }
    result
}

/// Takes a vec of values and a vec of origins and merges all values from the
/// given origins in the order of the `origins` vec. There, the last element in
/// `origins` has the highest priority.
/// TODO: does not work if values contains two values from the same origin.
pub fn merge_all_from<T>(
    values: Vec<WithOrigin<T>>,
    origins: &[Id<Origin>],
) -> Option<T>
where
    T: Mergable + Serialize + Clone,
{
    let mut result: Option<T> = None;
    for origin in origins {
        let next = values.iter().find(|v| v.origin == *origin).cloned();
        match (result, next) {
            (Some(current), Some(next)) => {
                result = Some(current.merge(next.content));
            }
            (None, Some(next)) => {
                result = Some(next.content);
            }
            (current, _) => {
                result = current;
            }
        }
    }
    result
}
