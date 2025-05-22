//! # Commands
//!
//! Commands for interacting with commands from the frontend

use crate::{
    database::{
        entity::{
            command_alias::{CommandAliasModel, CommandWithAliases},
            command_execution::CommandExecutionModel,
            command_log::CommandLogsModel,
            commands::{CommandModel, CreateCommand, UpdateCommand},
            shared::{ExecutionsQuery, LogsQuery, UpdateOrdering},
        },
        DbPool,
    },
    export::{self, ExportedCommandModel},
};
use anyhow::Context;
use tauri::{async_runtime::spawn_blocking, AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use super::CmdResult;

/// Get all commands
#[tauri::command]
pub async fn get_commands(db: State<'_, DbPool>) -> CmdResult<Vec<CommandModel>> {
    let db = db.inner();
    let commands = CommandModel::all(db).await?;
    Ok(commands)
}

/// Get a specific command by ID
#[tauri::command]
pub async fn get_command_by_id(
    command_id: Uuid,
    db: State<'_, DbPool>,
) -> CmdResult<Option<CommandWithAliases>> {
    let db = db.inner();
    let command = match CommandModel::get_by_id(db, command_id).await? {
        Some(value) => value,
        None => return Ok(None),
    };
    let aliases = CommandAliasModel::get_aliases(db, command.id).await?;

    Ok(Some(CommandWithAliases { command, aliases }))
}

/// Create a new command
#[tauri::command]
pub async fn create_command(
    create: CreateCommand,
    db: State<'_, DbPool>,
) -> CmdResult<CommandWithAliases> {
    let db = db.inner();
    let command = CommandModel::create(db, create).await?;
    let aliases = CommandAliasModel::get_aliases(db, command.id).await?;

    Ok(CommandWithAliases { command, aliases })
}

/// Update an existing command
#[tauri::command]
pub async fn update_command(
    command_id: Uuid,
    update: UpdateCommand,
    db: State<'_, DbPool>,
) -> CmdResult<CommandWithAliases> {
    let db = db.inner();
    let mut command = CommandModel::get_by_id(db, command_id)
        .await?
        .context("command not found")?;

    command.update(db, update).await?;

    let aliases = CommandAliasModel::get_aliases(db, command.id).await?;
    Ok(CommandWithAliases { command, aliases })
}

/// Delete a command
#[tauri::command]
pub async fn delete_command(command_id: Uuid, db: State<'_, DbPool>) -> CmdResult<()> {
    let db = db.inner();
    let command = CommandModel::get_by_id(db, command_id)
        .await?
        .context("command not found")?;
    command.delete(db).await?;
    Ok(())
}

/// Get logs of a command
#[tauri::command]
pub async fn get_command_logs(
    command_id: Uuid,
    query: LogsQuery,
    db: State<'_, DbPool>,
) -> CmdResult<Vec<CommandLogsModel>> {
    let db = db.inner();
    let logs = CommandLogsModel::query(db, command_id, query).await?;

    Ok(logs)
}

#[tauri::command]
pub async fn delete_command_logs(log_ids: Vec<Uuid>, db: State<'_, DbPool>) -> CmdResult<()> {
    let db = db.inner();

    CommandLogsModel::delete_by_ids(db, &log_ids).await?;

    Ok(())
}

#[tauri::command]
pub async fn update_command_orderings(
    update: Vec<UpdateOrdering>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    CommandModel::update_order(db, update).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_command_executions(
    command_id: Uuid,
    query: ExecutionsQuery,
    db: State<'_, DbPool>,
) -> CmdResult<Vec<CommandExecutionModel>> {
    let db = db.inner();
    let executions = CommandExecutionModel::query(db, command_id, query).await?;

    Ok(executions)
}

#[tauri::command]
pub async fn delete_command_executions(
    execution_ids: Vec<Uuid>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();

    CommandExecutionModel::delete_by_ids(db, &execution_ids).await?;

    Ok(())
}

#[tauri::command]
pub async fn export_commands(
    command_ids: Vec<Uuid>,
    app: AppHandle,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    let commands = export::export_commands(db, &command_ids).await?;
    let data = serde_json::to_vec_pretty(&commands)?;

    let file_path = spawn_blocking(move || {
        app.dialog()
            .file()
            .set_file_name("exported-commands.json")
            .add_filter("JSON", &["json"])
            .blocking_save_file()
    })
    .await?;

    let file_path = match file_path {
        Some(value) => value,
        None => return Ok(()),
    };

    let path = match file_path.as_path() {
        Some(value) => value,
        None => return Ok(()),
    };

    tokio::fs::write(path, &data).await?;

    Ok(())
}

#[tauri::command]
pub async fn import_commands(
    commands: Vec<ExportedCommandModel>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    export::import_commands(db, commands).await?;
    Ok(())
}
