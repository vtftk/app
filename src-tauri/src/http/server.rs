//! # Server
//!
//! Internal server for handling OAuth responses and serving the app overlay HTML

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use super::routes;
use crate::events::EventRecvHandle;
use crate::state::app_data::AppDataStore;
use crate::state::runtime_app_data::RuntimeAppDataStore;
use crate::twitch::manager::TwitchManager;
use anyhow::Context;
use axum::Extension;
use sea_orm::DatabaseConnection;
use tauri::AppHandle;
use tower_http::cors::CorsLayer;

pub async fn start(
    db: DatabaseConnection,
    event_handle: EventRecvHandle,
    app_handle: AppHandle,
    twitch_manager: Arc<TwitchManager>,
    app_data: AppDataStore,
    runtime_app_data: RuntimeAppDataStore,
) -> anyhow::Result<()> {
    let port = {
        let app_data = &*app_data.read().await;
        app_data.main_config.get_http_port()
    };

    // build our application with a single route
    let app = routes::router()
        .layer(Extension(db))
        .layer(Extension(event_handle))
        .layer(Extension(app_handle))
        .layer(Extension(twitch_manager))
        .layer(Extension(app_data))
        .layer(Extension(runtime_app_data))
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
