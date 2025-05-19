use crate::{
    commands::CmdResult,
    database::{
        entity::{
            app_data::{AppData, AppDataModel},
            chat_history::ChatHistoryModel,
            command_execution::CommandExecutionModel,
            command_log::CommandLogsModel,
            event_execution::EventExecutionModel,
            event_log::EventLogsModel,
        },
        DbPool,
    },
    http::ServerPort,
    overlay::{OverlayData, OverlayDataStore, OverlayMessage, OverlayMessageSender},
    storage::{Storage, StorageFolder},
};
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
pub fn get_overlay_url(port_state: State<'_, ServerPort>) -> CmdResult<String> {
    let http_port = port_state.inner().0;
    Ok(format!("http://localhost:{}/overlay", http_port))
}

/// Obtains the current app data state
#[tauri::command]
pub async fn get_app_data(db: tauri::State<'_, DbPool>) -> CmdResult<AppData> {
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
    db: tauri::State<'_, DbPool>,
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
pub async fn get_chat_history_estimate_size(db: tauri::State<'_, DbPool>) -> CmdResult<u32> {
    Ok(ChatHistoryModel::estimate_size(db.inner()).await?)
}

/// Get the estimated size of executions in bytes
#[tauri::command]
pub async fn get_executions_estimate_size(db: tauri::State<'_, DbPool>) -> CmdResult<u32> {
    let (command_size, event_size) = try_join!(
        CommandExecutionModel::estimated_size(db.inner()),
        EventExecutionModel::estimated_size(db.inner())
    )?;

    Ok(command_size.saturating_add(event_size))
}

/// Get the estimated size of logs in bytes
#[tauri::command]
pub async fn get_logs_estimate_size(db: tauri::State<'_, DbPool>) -> CmdResult<u32> {
    let (command_size, event_size) = try_join!(
        CommandLogsModel::estimated_size(db.inner()),
        EventLogsModel::estimated_size(db.inner())
    )?;

    Ok(command_size.saturating_add(event_size))
}

/// Get the current HTTP server port
#[tauri::command]
pub fn get_http_port(port_state: State<'_, ServerPort>) -> CmdResult<u16> {
    Ok(port_state.inner().0)
}
