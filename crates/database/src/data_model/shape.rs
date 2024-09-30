use model::shape::ShapePoint;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ShapePointRow {
    pub id: i32,
    pub sequence: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub distance: Option<f64>,
}

impl ShapePointRow {
    pub fn to_model(self) -> ShapePoint {
        ShapePoint {
            latitude: self.latitude,
            longitude: self.longitude,
            distance: self.distance,
        }
    }
}
