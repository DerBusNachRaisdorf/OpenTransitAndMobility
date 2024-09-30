/// A location. E.g., a train station or a bus stop.
/// Table: locations
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Location {
    pub id: String,
    pub name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// A name alias for a station.
/// Table: `location_aliases`
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocationAlias {
    pub location_id: String,
    pub alias: String,
}

/// Maps an eva number to a location.
/// Table: `eva_numbers`
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EvaNumber {
    pub eva_number: u32,
    pub location_id: String,
}

/// Maps a ril100 identifier to a location.
/// Table: `ril100_identifiers`
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Ril100Identifier {
    pub ril100_identifier: String,
    pub location_id: String,
}
