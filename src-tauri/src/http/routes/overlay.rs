use crate::http::models::UpdateRuntimeAppData;
use crate::overlay::{OverlayDataStore, OverlayEventStream, OverlayMessageReceiver, OVERLAY_PAGE};
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Extension,
};
use futures::Stream;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use std::convert::Infallible;

/// Embedded icon for VTube studio
const ICON: &[u8] = include_bytes!("../resources/128x128.png");

/// GET /overlay
///
/// HTML page for the overlay
pub async fn page() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/html")], OVERLAY_PAGE)
}

/// GET /overlay/icon
///
/// Icon for the overlay to provide to VTube studio when
/// authenticating
pub async fn icon() -> impl IntoResponse {
    ([(CONTENT_TYPE, "image/png")], ICON)
}

/// GET /overlay/events
///
/// EventSource for the overlay, when connected increases the overlay count and
/// provides overlay events
pub async fn handle_sse(
    Extension(overlay_msg_rx): Extension<OverlayMessageReceiver>,
    Extension(overlay_data): Extension<OverlayDataStore>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let overlay = overlay_data.create_overlay().await;

    Sse::new(OverlayEventStream::new(overlay, overlay_msg_rx)).keep_alive(KeepAlive::default())
}

/// PUT /overlay/data
///
/// Partially the current overlay data
pub async fn update_overlay_data(
    Extension(runtime_app_data): Extension<OverlayDataStore>,
    Json(req): Json<UpdateRuntimeAppData>,
) -> StatusCode {
    // Update the stored runtime data
    runtime_app_data
        .write(|runtime_app_data| {
            if let Some(model_id) = req.model_id {
                runtime_app_data.model_id = model_id;
            }

            if let Some(vtube_studio_connected) = req.vtube_studio_connected {
                runtime_app_data.vtube_studio_connected = vtube_studio_connected;
            }

            if let Some(vtube_studio_auth) = req.vtube_studio_auth {
                runtime_app_data.vtube_studio_auth = vtube_studio_auth;
            }

            if let Some(hotkeys) = req.hotkeys {
                runtime_app_data.hotkeys = hotkeys;
            }
        })
        .await;

    StatusCode::OK
}
