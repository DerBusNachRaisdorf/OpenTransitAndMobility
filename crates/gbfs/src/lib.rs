use model::{
    shared_mobility::{self, SharedMobilityStation},
    WithId,
};
use public_transport::{
    client::Client, database::Database, RequestError, RequestResult,
};
use serde::Deserialize;
use utility::id::Id;

pub mod collector;

#[derive(Debug, Clone, Deserialize)]
pub struct StationInformation {
    pub station_id: String,
    pub name: String,
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lon")]
    pub longitude: f64,
    pub capacity: u32,
    pub rental_uris: RentalUris,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalUris {
    pub android: Option<String>,
    pub ios: Option<String>,
    pub web: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StationStatus {
    pub station_id: String,
    pub num_bikes_available: u32,
    pub num_docks_available: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StationRespones<T> {
    pub stations: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

pub async fn update_station_status<D: Database>(
    client: Client<D>,
    url: &str,
) -> RequestResult<()> {
    let response: Response<StationRespones<StationStatus>> = reqwest::get(url)
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?
        .json()
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?;

    for status in response.data.stations {
        client
            .update_shared_mobility_station_status(
                &Id::new(status.station_id),
                Some(shared_mobility::Status {
                    num_bikes_available: status.num_bikes_available,
                    num_docks_available: status.num_docks_available,
                }),
            )
            .await?;
    }

    Ok(())
}

pub async fn insert_station_information<D: Database>(
    client: Client<D>,
    url: &str,
) -> RequestResult<()> {
    let response: Response<StationRespones<StationInformation>> = reqwest::get(url)
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?
        .json()
        .await
        .map_err(|why| RequestError::Other(Box::new(why)))?;

    client
        .put_shared_mobility_stations(
            response
                .data
                .stations
                .into_iter()
                .map(|station| {
                    WithId::new(
                        Id::new(station.station_id), // TODO!
                        SharedMobilityStation {
                            name: station.name,
                            latitude: station.latitude,
                            longitude: station.longitude,
                            capacity: station.capacity,
                            rental_uris: model::shared_mobility::RentalUris {
                                android: station.rental_uris.android,
                                ios: station.rental_uris.ios,
                                web: station.rental_uris.web,
                            },
                            status: None,
                        },
                    )
                })
                .collect::<Vec<_>>(),
        )
        .await?;

    Ok(())
}
