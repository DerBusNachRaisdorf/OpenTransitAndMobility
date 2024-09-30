use std::sync::Arc;

use crate::{
    client::{Accept, BahnApiClient},
    model::station_data::StationQuery,
    ApiError,
};

pub async fn get_station_data(
    client: Arc<BahnApiClient>,
    federal_state: &str,
) -> Result<StationQuery, ApiError> {
    /* fetch data */
    client
        .get(
            &format!("station-data/v2/stations?federalstate={}", federal_state),
            Accept::Json,
        )
        .await
}
