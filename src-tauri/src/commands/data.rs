use crate::{
    commands::CmdResult,
    database::entity::{
        app_data::{AppData, AppDataModel},
        chat_history::ChatHistoryModel,
        command_executions::CommandExecutionModel,
        command_logs::CommandLogsModel,
        event_executions::EventExecutionModel,
        event_logs::EventLogsModel,
    },
    overlay::{OverlayData, OverlayDataStore, OverlayMessage, OverlayMessageSender},
    storage::{Storage, StorageFolder},
};
use sea_orm::DatabaseConnection;
use tauri::State;
use tokio::try_join;

/// Requests that an active overlay update the current list
/// of hotkeys from VTube Studio
#[tauri::command]
pub fn update_hotkeys(event_sender: tauri::State<'_, OverlayMessageSender>) -> CmdResult<()> {
    event_sender.send(OverlayMessage::UpdateHotkeys)?;
    Ok(())
}

/// Obtains the current URL for the OBS overlay
#[tauri::command]
pub async fn get_overlay_url(db: tauri::State<'_, DatabaseConnection>) -> CmdResult<String> {
    let http_port = AppDataModel::get_http_port(db.inner()).await?;
    Ok(format!("http://localhost:{}/overlay", http_port))
}

/// Obtains the current app data state
#[tauri::command]
pub async fn get_app_data(db: tauri::State<'_, DatabaseConnection>) -> CmdResult<AppData> {
    Ok(AppDataModel::get_or_default(db.inner()).await?)
}

/// Obtains the current runtime app data
#[tauri::command]
pub async fn get_runtime_app_data(
    runtime_app_data: tauri::State<'_, OverlayDataStore>,
) -> CmdResult<OverlayData> {
    Ok(runtime_app_data.read().await.clone())
}

/// Updates the current app data
#[tauri::command]
pub async fn set_app_data(
    app_data: AppData,
    db: tauri::State<'_, DatabaseConnection>,
    event_sender: tauri::State<'_, OverlayMessageSender>,
) -> CmdResult<bool> {
    let model = AppDataModel::set(db.inner(), app_data).await?;

    // Inform the overlay of the new app data
    _ = event_sender.send(OverlayMessage::ConfigUpdated {
        config: Box::new(model.data.overlay),
    });

    Ok(true)
}

#[tauri::command]
pub async fn upload_file(
    folder: StorageFolder,
    name: String,
    data: Vec<u8>,
    storage: State<'_, Storage>,
) -> CmdResult<String> {
    let url = storage.upload_file(folder, name, data).await?;
    Ok(url)
}

/// Get the estimated size of chat history in bytes
#[tauri::command]
pub async fn get_chat_history_estimate_size(
    db: tauri::State<'_, DatabaseConnection>,
) -> CmdResult<u32> {
    Ok(ChatHistoryModel::estimate_size(db.inner()).await?)
}

/// Get the estimated size of executions in bytes
#[tauri::command]
pub async fn get_executions_estimate_size(
    db: tauri::State<'_, DatabaseConnection>,
) -> CmdResult<u32> {
    let (command_size, event_size) = try_join!(
        CommandExecutionModel::estimate_size(db.inner()),
        EventExecutionModel::estimate_size(db.inner())
    )?;

    Ok(command_size.saturating_add(event_size))
}

/// Get the estimated size of logs in bytes
#[tauri::command]
pub async fn get_logs_estimate_size(db: tauri::State<'_, DatabaseConnection>) -> CmdResult<u32> {
    let (command_size, event_size) = try_join!(
        CommandLogsModel::estimate_size(db.inner()),
        EventLogsModel::estimate_size(db.inner())
    )?;

    Ok(command_size.saturating_add(event_size))
}
