//! # Server
//!
//! Internal server for handling OAuth responses and serving the app overlay HTML

use crate::{
    database::{entity::app_data::AppDataModel, DbPool},
    overlay::{OverlayDataStore, OverlayMessageReceiver, OverlayMessageSender},
    storage::Storage,
    twitch::manager::Twitch,
};
use anyhow::Context;
use axum::Extension;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tauri::AppHandle;
use tower_http::cors::CorsLayer;

mod error;
pub mod models;
pub mod routes;

pub async fn start_http_server(
    db: DbPool,
    overlay_tx: OverlayMessageSender,
    overlay_rx: OverlayMessageReceiver,
    app_handle: AppHandle,
    twitch: Twitch,
    overlay_data: OverlayDataStore,
    storage: Storage,
) -> anyhow::Result<()> {
    let port = AppDataModel::get_http_port(&db).await?;

    // build our application with a single route
    let app = routes::router()
        .layer(Extension(db))
        .layer(Extension(overlay_tx))
        .layer(Extension(overlay_rx))
        .layer(Extension(app_handle))
        .layer(Extension(twitch))
        .layer(Extension(overlay_data))
        .layer(Extension(storage))
        .layer(CorsLayer::very_permissive());

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind http server socket")?;
    axum::serve(listener, app)
        .await
        .context("error while serving")?;

    Ok(())
}
