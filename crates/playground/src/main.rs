use std::sync::Arc;

use deutsche_bahn::{
    client::{BahnApiClient, BahnApiCredentials},
    station_data::get_station_data,
};

#[tokio::main]
async fn main() {
    let credentials = BahnApiCredentials {
        client_id: "0c31f17ab92c35f90caf47c204fac269".to_owned(),
        client_secret: "a0aab810372228c557eadb3589e0b79d".to_owned(),
        rate_limit_per_minute: Some(60),
        proxy: None,
    };
    let client = Arc::new(BahnApiClient::new(&credentials));
    let result = get_station_data(client, "schleswig-holstein")
        .await
        .unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    println!("json: {}", json);
}
