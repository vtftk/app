use crate::database::entity::app_data::{AppDataModel, OverlayConfig};
use crate::database::DbPool;
use crate::http::error::{DynHttpError, HttpResult};
use crate::http::models::UpdateRuntimeAppData;
use crate::overlay::{
    OverlayDataStore, OverlayEventStream, OverlayMessage, OverlayMessageReceiver,
    OverlayMessageSender, OVERLAY_PAGE,
};
use anyhow::Context;
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

/// GET /overlay/config
///
/// Get the overlay configuration data
pub async fn get_overlay_config(Extension(db): Extension<DbPool>) -> HttpResult<OverlayConfig> {
    let data = AppDataModel::get_or_default(&db).await?;
    Ok(Json(data.overlay))
}

/// POST /overlay/events
///
/// Emit an event to the overlay
pub async fn emit_event(
    Extension(overlay_tx): Extension<OverlayMessageSender>,
    Json(req): Json<OverlayMessage>,
) -> Result<StatusCode, DynHttpError> {
    overlay_tx.send(req).context("failed to message overlay")?;
    Ok(StatusCode::NO_CONTENT)
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
    Extension(overlay_data): Extension<OverlayDataStore>,
    Json(req): Json<UpdateRuntimeAppData>,
) -> StatusCode {
    // Update the stored runtime data
    overlay_data
        .write(|overlay_data| {
            if let Some(model_id) = req.model_id {
                overlay_data.model_id = model_id;
            }

            if let Some(vtube_studio_connected) = req.vtube_studio_connected {
                overlay_data.vtube_studio_connected = vtube_studio_connected;
            }

            if let Some(vtube_studio_auth) = req.vtube_studio_auth {
                overlay_data.vtube_studio_auth = vtube_studio_auth;
            }

            if let Some(hotkeys) = req.hotkeys {
                overlay_data.hotkeys = hotkeys;
            }
        })
        .await;

    StatusCode::OK
}
