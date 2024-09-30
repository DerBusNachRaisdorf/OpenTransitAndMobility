use database::{DatabaseConnectionInfo, PgDatabase};
use public_transport::server::Server;
use web::{start_web_server, WebState};

#[tokio::main]
async fn main() {
    env_logger::init();

    // database
    let database_connection_info = DatabaseConnectionInfo::from_env()
        .expect("expected database connection info in env.");
    let database = PgDatabase::connect(database_connection_info)
        .await
        .expect("could not connect to database.");

    // server
    let server = Server::new(database.clone());
    server
        .collectors::<gtfs::collector::ScheduleCollector>()
        .await
        .unwrap();
    server
        .collectors::<gtfs::collector::RealtimeCollector>()
        .await
        .unwrap();
    server
        .collectors::<gbfs::collector::StationsCollector>()
        .await
        .unwrap();
    server
        .collectors::<gbfs::collector::StatusCollector>()
        .await
        .unwrap();
    server
        .collectors::<deutsche_bahn::collector::DeutscheBahnCollector>()
        .await
        .unwrap();

    /*
    // gtfs nah.sh
    let gtfs_sh_id = server.origin("GTFS NAH.SH", 1).await.unwrap();
    server.collector(&gtfs_sh_id, || {
        gtfs::collector::ScheduleCollector::new(
            "https://www.connect-info.net/opendata/gtfs/nah.sh/rjqfrkqhgu",
        )
    });
    server.collector(&gtfs_sh_id, || {
        gtfs::collector::RealtimeCollector::new(
            "https://gtfsr.vbn.de/DluWlw1jMRKr",
            Duration::from_secs(60),
        )
    });

    // gbfs donkey kiel
    let gbfs_donkey_kiel_id = server.origin("GBFS Donkey Kiel", 2).await.unwrap();
    server.collector(&gbfs_donkey_kiel_id, || {
        gbfs::collector::StatusCollector::new(
            "https://stables.donkey.bike/api/public/gbfs/2/donkey_kiel/en/station_status.json"
        )
    });
    */

    // web server
    let web_future = start_web_server(WebState {
        transit_client: server.client("REST API"),
    });

    let _ = web_future.await;
}
