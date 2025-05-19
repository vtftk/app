//! # Server
//!
//! Internal server for handling OAuth responses and serving the app overlay HTML

use crate::{
    database::DbPool,
    overlay::{OverlayDataStore, OverlayMessageReceiver, OverlayMessageSender},
    storage::Storage,
    twitch::manager::Twitch,
};
use anyhow::Context;
use axum::Extension;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tauri::AppHandle;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod error;
pub mod models;
pub mod routes;

#[derive(Clone, Copy)]
pub struct ServerPort(pub u16);

pub async fn create_http_socket(port: u16) -> std::io::Result<TcpListener> {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));
    TcpListener::bind(addr).await
}

pub struct HttpExtensions {
    pub db: DbPool,
    pub overlay_tx: OverlayMessageSender,
    pub overlay_rx: OverlayMessageReceiver,
    pub app_handle: AppHandle,
    pub twitch: Twitch,
    pub overlay_data: OverlayDataStore,
    pub storage: Storage,
}

pub async fn start_http_server(
    listener: TcpListener,
    extensions: HttpExtensions,
) -> anyhow::Result<()> {
    // build our application with a single route
    let app = routes::router()
        .layer(Extension(extensions.db))
        .layer(Extension(extensions.overlay_tx))
        .layer(Extension(extensions.overlay_rx))
        .layer(Extension(extensions.app_handle))
        .layer(Extension(extensions.twitch))
        .layer(Extension(extensions.overlay_data))
        .layer(Extension(extensions.storage))
        .layer(CorsLayer::very_permissive());

    axum::serve(listener, app)
        .await
        .context("error while serving")?;

    Ok(())
}
