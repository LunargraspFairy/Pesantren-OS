use std::{convert::Infallible, time::Duration};
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::Stream;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{auth::CurrentUser, error::AppError, AppState};

pub async fn sse_dormitory_stream(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(dorm_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    user.require_admin_or_musyrif_for_dorm(&dorm_id)?;

    let rx = state.sse.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |res| match res {
        Ok(msg) if msg.target_dormitory_id.as_deref() == Some(dorm_id.as_str()) => {
            let json_payload = serde_json::to_string(&msg).unwrap_or_default();
            Some(Ok(Event::default().event(msg.event_name).data(json_payload)))
        }
        _ => None,
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

pub async fn sse_public_flow_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.sse.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| match res {
        Ok(msg) if msg.event_name == "public_flow_updated" => {
            let json_payload = serde_json::to_string(&msg).unwrap_or_default();
            Some(Ok(Event::default().event("public_flow_updated").data(json_payload)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
