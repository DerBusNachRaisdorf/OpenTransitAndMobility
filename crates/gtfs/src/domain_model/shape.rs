use utility::id::{HasId, Id};

use crate::data_model::{Latitude, Longitude};

pub struct Shape {
    pub id: Id<Shape>,
    pub points: Vec<ShapePoint>,
}

impl HasId for Shape {
    type IdType = String;
}

pub struct ShapePoint {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub distance_traveled: Option<u32>,
}
