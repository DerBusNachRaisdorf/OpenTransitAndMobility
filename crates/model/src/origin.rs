use std::fmt::Debug;

use serde::Serialize;
use utility::id::{HasId, Id};

#[derive(Debug, Clone, Serialize)]
pub struct Origin {
    pub name: String,
    pub priority: i32,
}

impl HasId for Origin {
    type IdType = String;
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginalIdMapping<S>
where
    S: HasId,
    S::IdType: Debug + Clone + Serialize,
{
    pub origin: Id<Origin>,
    pub original_id: String,
    pub id: Id<S>,
}
