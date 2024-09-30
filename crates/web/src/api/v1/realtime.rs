use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, on},
    Router,
};
use axum_extra::TypedHeader;
use chrono::Local;
use futures::stream::{self, Stream};
use model::{trip_update::TripUpdate, DateTimeRange, WithId};
use serde::Serialize;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::trace::TraceLayer;

use crate::{
    common::{route_not_found, METHOD_FILTER_ALL},
    WebState,
};

use super::TripsNearbyQuery;

macro_rules! resource {
    ($($arg:tt)*) => {
        crate::api::v1::resource!("/realtime{}", format_args!($($arg)*))
    };
}
pub(crate) use resource;

pub(crate) fn routes(state: WebState) -> Router {
    Router::new()
        .route("/nearby", get(sse_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .fallback_service(on(METHOD_FILTER_ALL, route_not_found))
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateEvent {
    trip_updates: Vec<WithId<TripUpdate>>,
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(WebState { transit_client, .. }): State<WebState>,
    Query(params): Query<TripsNearbyQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let origins = transit_client.get_origin_ids().await.expect("origins");
    let radius = params.radius.unwrap_or(0.05);
    let start = params.start.unwrap_or(Local::now());
    let end = params.end.unwrap_or(start + chrono::Duration::hours(1));

    let stops = transit_client
        .find_nearby(params.latitude, params.longitude, radius, &origins)
        .await
        .expect("stops");

    let stop_ids = stops
        .iter()
        .map(|stop| &stop.content.id)
        .collect::<Vec<_>>();

    let trip_ids = transit_client
        .get_all_trips_via_stops(&stop_ids, start, end, &origins)
        .await
        .expect("trips")
        .into_iter()
        .map(|trip| trip.id)
        .collect::<Vec<_>>();

    let stream = stream::unfold((), move |()| {
        let client = transit_client.clone();
        let origins = origins.clone();
        let trip_ids = trip_ids.clone();
        async move {
            let updates = client
                .get_realtime_for_trips_in_range(
                    &trip_ids,
                    DateTimeRange::new(start, end),
                    &origins,
                )
                .await
                .unwrap_or(vec![]); // TODO: error handling
            let event_data = UpdateEvent {
                trip_updates: updates,
            };
            let event = Event::default()
                .json_data(event_data)
                .expect("TODO: error handling");
            Some((event, ()))
        }
    })
    .map(Ok)
    .throttle(Duration::from_secs(10));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
