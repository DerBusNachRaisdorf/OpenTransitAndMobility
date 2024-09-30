use std::fmt::Debug;

use model::{origin::Origin, WithId, WithOrigin};
use serde::Serialize;
use utility::id::{HasId, Id};

pub mod agency;
pub mod calendar;
pub mod calendar_exception;
pub mod collector;
pub mod line;
pub mod location;
pub mod origin;
pub mod shared_mobility;
pub mod stop;
pub mod trip;
pub mod trip_update;
pub mod shape;

pub type Result<O> = core::result::Result<O, sqlx::Error>;

pub trait DatabaseRow {
    type Model: Serialize + HasId;

    fn get_id(&self) -> Id<Self::Model>;
    fn get_origin(&self) -> Id<Origin>;
    fn to_model(self) -> Self::Model;
    fn from_model(model: WithOrigin<Self::Model>) -> Self;
}

pub fn with_origins_and_ids<R: DatabaseRow>(
    rows: Vec<R>,
) -> Vec<WithOrigin<WithId<R::Model>>>
where
    <R::Model as HasId>::IdType: Debug + Clone + Serialize,
{
    rows.into_iter()
        .map(|row| with_origin_and_id(row))
        .collect::<Vec<_>>()
}

pub fn with_origin_and_id<R: DatabaseRow>(row: R) -> WithOrigin<WithId<R::Model>>
where
    <R::Model as HasId>::IdType: Debug + Clone + Serialize,
{
    WithOrigin::new(row.get_origin(), WithId::new(row.get_id(), row.to_model()))
}

pub fn with_origins<R: DatabaseRow>(rows: Vec<R>) -> Vec<WithOrigin<R::Model>> {
    rows.into_iter()
        .map(|row| with_origin(row))
        .collect::<Vec<_>>()
}

pub fn with_origin<R: DatabaseRow>(row: R) -> WithOrigin<R::Model> {
    WithOrigin::new(row.get_origin(), row.to_model())
}

pub fn with_ids<R: DatabaseRow>(rows: Vec<R>) -> Vec<WithId<R::Model>>
where
    <R::Model as HasId>::IdType: Debug + Clone + Serialize,
{
    rows.into_iter().map(|row| with_id(row)).collect::<Vec<_>>()
}

pub fn with_id<R: DatabaseRow>(row: R) -> WithId<R::Model>
where
    <R::Model as HasId>::IdType: Debug + Clone + Serialize,
{
    WithId::new(row.get_id(), row.to_model())
}
