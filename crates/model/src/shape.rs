use serde::{Deserialize, Serialize};
use utility::id::HasId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapePoint {
    pub latitude: f64,
    pub longitude: f64,
    pub distance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub points: Vec<ShapePoint>,
}

impl HasId for Shape {
    type IdType = i32;
}
