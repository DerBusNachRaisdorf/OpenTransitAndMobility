pub const EARTH_RADIUS_KM: f64 = 6371.0;

fn to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn to_degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

pub fn calculate_bounding_box(
    lat: f64,
    lon: f64,
    radius_km: f64,
) -> ((f64, f64), (f64, f64)) {
    // Convert latitude and longitude from degrees to radians
    let lat_rad = to_radians(lat);
    let lon_rad = to_radians(lon);

    // Latitude bounds
    let min_lat = lat_rad - radius_km / EARTH_RADIUS_KM;
    let max_lat = lat_rad + radius_km / EARTH_RADIUS_KM;

    // Longitude bounds (adjusted by latitude)
    let min_lon = lon_rad - radius_km / (EARTH_RADIUS_KM * lat_rad.cos());
    let max_lon = lon_rad + radius_km / (EARTH_RADIUS_KM * lat_rad.cos());

    // Convert bounds back to degrees
    let min_lat_deg = to_degrees(min_lat);
    let max_lat_deg = to_degrees(max_lat);
    let min_lon_deg = to_degrees(min_lon);
    let max_lon_deg = to_degrees(max_lon);

    ((min_lat_deg, min_lon_deg), (max_lat_deg, max_lon_deg))
}

pub fn haversine_distance(
    latitude_1: f64,
    longitude_1: f64,
    latitude2: f64,
    longitude_2: f64,
) -> f64 {
    let lat1_rad = to_radians(latitude_1);
    let lon1_rad = to_radians(longitude_1);
    let lat2_rad = to_radians(latitude2);
    let lon2_rad = to_radians(longitude_2);

    let dlat = lat2_rad - lat1_rad;
    let dlon = lon2_rad - lon1_rad;

    let a = (dlat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}
