use super::CmdResult;
use crate::events::outcome::resolve_items;
use crate::overlay::{OverlayMessage, OverlayMessageSender, ThrowItemConfig, ThrowItemMessage};
use anyhow::Context;
use log::debug;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};
use tauri::State;
use tokio::{net::UdpSocket, time::timeout};
use uuid::Uuid;

/// Plays a test throw item event
#[tauri::command]
pub async fn test_throw(
    item_ids: Vec<Uuid>,
    amount: Option<i64>,
    db: State<'_, DatabaseConnection>,
    event_sender: State<'_, OverlayMessageSender>,
) -> CmdResult<()> {
    let db = db.inner();
    let items = resolve_items(db, &item_ids).await?;

    event_sender.send(OverlayMessage::ThrowItem(ThrowItemMessage {
        items,
        config: ThrowItemConfig::All {
            amount: amount.unwrap_or(1),
        },
    }))?;

    Ok(())
}

/// Plays a test throw item event
#[tauri::command]
pub async fn test_throw_barrage(
    item_ids: Vec<Uuid>,
    amount_per_throw: u32,
    amount: i64,
    frequency: u32,
    db: State<'_, DatabaseConnection>,
    event_sender: State<'_, OverlayMessageSender>,
) -> CmdResult<()> {
    let db = db.inner();
    let items = resolve_items(db, &item_ids).await?;

    event_sender.send(OverlayMessage::ThrowItem(ThrowItemMessage {
        items,
        config: ThrowItemConfig::Barrage {
            amount_per_throw,
            amount,
            frequency,
        },
    }))?;

    Ok(())
}

/// Attempts to detect a locally running VTube studio instance by using
/// the "API Server Discovery (UDP)" protocol
#[tauri::command]
pub async fn detect_vtube_studio() -> CmdResult<VTubeStudioBroadcast> {
    let discover_future = vtube_studio_detect_server();
    let future = timeout(Duration::from_secs(5), discover_future)
        .await
        .context("timeout while discovering")??;
    Ok(future)
}

/// Small UDP socket server to run on the discovery port
async fn vtube_studio_detect_server() -> anyhow::Result<VTubeStudioBroadcast> {
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 47779));
    let socket = UdpSocket::bind(address)
        .await
        .context("failed to bind socket")?;

    let mut buf = vec![0; 1024]; // Buffer to store incoming data

    loop {
        let count = socket.recv(&mut buf).await?;
        let msg = &buf[..count];
        match serde_json::from_slice::<VTubeStudioBroadcast>(msg) {
            Ok(broadcast) => {
                debug!("recv vtube studio broadcast message: {:?}", broadcast);
                return Ok(broadcast);
            }
            Err(err) => {
                debug!("discarding junk broadcast message: {:?}", err);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VTubeStudioBroadcast {
    api_name: String,
    api_version: String,
    timestamp: u64,
    message_type: String,
    #[serde(rename = "requestID")]
    request_id: String,
    data: BroadcastData,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastData {
    active: bool,
    port: u16,
    #[serde(rename = "instanceID")]
    instance_id: String,
    window_title: String,
}
