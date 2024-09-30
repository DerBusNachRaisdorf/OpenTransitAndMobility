use std::fmt::Debug;

use model::origin::OriginalIdMapping;
use serde::Serialize;
use sqlx::prelude::FromRow;
use utility::id::{HasId, Id};

#[derive(Debug, Clone, FromRow)]
pub struct OriginRow {
    pub id: String,
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct OriginalIdMappingRow<T> {
    pub origin: String,
    pub original_id: String,
    pub id: T,
}

impl<T> OriginalIdMappingRow<T> {
    pub fn to_model<S>(self) -> OriginalIdMapping<S>
    where
        S: HasId,
        S::IdType: Debug + Clone + Serialize + From<T>,
    {
        OriginalIdMapping {
            origin: Id::new(self.origin),
            original_id: self.original_id,
            id: Id::new(self.id.into()),
        }
    }
}
