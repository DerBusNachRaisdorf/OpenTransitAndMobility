use serde::{Deserialize, Serialize};
use utility::id::{HasId, Id};

use super::{Latitude, Longitude};

/// Rules for mapping vehicle travel paths, sometimes referred to as route alignments.
///
/// Primary key `(shape_id, shape_pt_sequence)`
///
/// Shapes describe the path that a vehicle travels along a route alignment, and are
/// defined in the file shapes.txt. Shapes are associated with Trips, and consist of a
/// sequence of points through which the vehicle passes in order. Shapes do not need
/// to intercept the location of Stops exactly, but all Stops on a trip should lie
/// within a small distance of the shape for that trip, i.e. close to straight line
/// segments connecting the shape points.
///
/// See <https://gtfs.org/schedule/reference/#shapestxt>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapesRow {
    /// Identifies a shape.
    pub shape_id: Id<ShapesRow>, // TODO: id type!!

    /// Latitude of a shape point. Each record in shapes.txt represents a shape point
    /// used to define the shape.
    #[serde(rename = "shape_pt_lat")]
    pub point_latitude: Latitude,

    /// Longitude of a shape point.
    #[serde(rename = "shape_pt_lon")]
    pub point_longitude: Longitude,

    /// Sequence in which the shape points connect to form the shape. Values must
    /// increase along the trip but do not need to be consecutive.
    ///
    /// # Example
    ///
    /// If the shape "A_shp" has three points in its definition, the shapes.txt file
    /// might contain these records to define the shape:
    ///
    /// - `shape_id`, `shape_pt_lat`, `shape_pt_lon`, `shape_pt_sequence`
    /// - `A_shp`,    `37.61956`,     `-122.48161`,   `0`
    /// - `A_shp`,    `37.64430`,     `-122.41070`,   `6`
    /// - `A_shp`,    `37.65863`,     `-122.30839`,   `11`
    ///
    /// Type specified as Non-negative integer in GTFS Schedule reference.
    #[serde(rename = "shape_pt_sequence")]
    pub point_sequence: u32,

    /// Actual distance traveled along the shape from the first shape point to the
    /// point specified in this record. Used by trip planners to show the correct
    /// portion of the shape on a map. Values must increase along with
    /// `shape_pt_sequence`; they must not be used to show reverse travel along a
    /// route. Distance units must be consistent with those used in stop_times.txt.
    ///
    /// Recommended for routes that have looping or inlining (the vehicle crosses or
    /// travels over the same portion of alignment in one trip).
    ///
    /// ![](https://gtfs.org/assets/inlining.svg)
    ///
    /// If a vehicle retraces or crosses the route alignment at points in the course
    /// of a trip, `shape_dist_traveled` is important to clarify how portions of the
    /// points in shapes.txt line up correspond with records in stop_times.txt.
    ///
    /// # Example
    ///
    /// If a bus travels along the three points defined above for `A_shp`, the
    /// additional `shape_dist_traveled` values (shown here in kilometers) would look
    /// like this:
    ///
    /// - `shape_id`, `shape_pt_lat`, `shape_pt_lon`, `shape_pt_sequence`, `shape_dist_traveled`
    /// - `A_shp`,    `37.61956`,     `-122.48161`,   `0`,                 `0`
    /// - `A_shp`,    `37.64430`,     `-122.41070`,   `6`,                 `6.8310`
    /// - `A_shp`,    `37.65863`,     `-122.30839`,   `11`,                `15.8765`
    ///
    /// Type specified as Non-negative integer in GTFS Schedule reference.
    #[serde(rename = "shape_dist_traveled")]
    pub distance_traveled: Option<f64>,
}

impl HasId for ShapesRow {
    type IdType = String;
}
