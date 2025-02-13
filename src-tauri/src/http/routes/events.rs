use std::{
    convert::Infallible,
    pin::Pin,
    task::{ready, Poll},
};

use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Extension,
};
use futures::Stream;
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    events::{OverlayMessage, EventRecvHandle},
    state::runtime_app_data::RuntimeAppDataStore,
};

pub async fn handle_sse(
    Extension(event_handle): Extension<EventRecvHandle>,
    Extension(runtime_app_data): Extension<RuntimeAppDataStore>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Increase number of active overlays
    runtime_app_data
        .write(|app_data| {
            app_data.active_overlay_count = app_data.active_overlay_count.saturating_add(1);
        })
        .await;

    let stream = BroadcastStream::new(event_handle.0);

    Sse::new(OverlayEventStream {
        runtime_app_data,
        stream,
    })
    .keep_alive(KeepAlive::default())
}

/// Wrapper around the event handle to receive events for the runtime  
///
/// Once the connection is dropped the number of active overlays is
/// decreased and associated state is dropped if overlays reaches zero
pub struct OverlayEventStream {
    runtime_app_data: RuntimeAppDataStore,
    stream: BroadcastStream<OverlayMessage>,
}

impl Stream for OverlayEventStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);
        let event = match ready!(stream.poll_next(cx)) {
            Some(Ok(value)) => value,
            _ => return Poll::Ready(None),
        };

        let event = match Event::default().json_data(event) {
            Ok(value) => value,
            _ => return Poll::Ready(None),
        };

        Poll::Ready(Some(Ok(event)))
    }
}

impl Drop for OverlayEventStream {
    fn drop(&mut self) {
        let runtime_app_data = self.runtime_app_data.clone();

        // Decrease the counter of active streams
        tauri::async_runtime::spawn(async move {
            let runtime_app_data = runtime_app_data;

            runtime_app_data
                .write(|app_data| {
                    app_data.active_overlay_count = app_data.active_overlay_count.saturating_sub(1);

                    // No longer connected to vtube studio or model
                    if app_data.active_overlay_count == 0 {
                        app_data.vtube_studio_connected = false;
                        app_data.vtube_studio_auth = false;
                        app_data.model_id = None;
                    }
                })
                .await;
        });
    }
}
