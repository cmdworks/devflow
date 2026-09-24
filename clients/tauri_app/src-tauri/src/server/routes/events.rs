use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast;

use crate::state::AppState;

pub async fn handle_events_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_bus.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(evt) => {
                    if let Ok(json_str) = serde_json::to_string(&evt) {
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
