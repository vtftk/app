//! # Commands
//!
//! Commands for interacting with commands from the frontend

use crate::database::{
    entity::{
        commands::{
            CommandExecutionModel, CommandLogsModel, CommandModel, CommandWithAliases,
            CreateCommand, UpdateCommand,
        },
        shared::{ExecutionsQuery, LogsQuery, UpdateOrdering},
    },
    DbPool,
};
use anyhow::Context;
use tauri::State;
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
    let command = CommandModel::get_by_id_with_aliases(db, command_id).await?;
    Ok(command)
}

/// Create a new command
#[tauri::command]
pub async fn create_command(
    create: CreateCommand,
    db: State<'_, DbPool>,
) -> CmdResult<CommandWithAliases> {
    let db = db.inner();
    let command = CommandModel::create(db, create).await?;
    let aliases = command.get_aliases(db).await?;

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

    let aliases = command.get_aliases(db).await?;
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
    let command = CommandModel::get_by_id(db, command_id)
        .await?
        .context("command not found")?;
    let logs = command.get_logs(db, query).await?;

    Ok(logs)
}

#[tauri::command]
pub async fn delete_command_logs(log_ids: Vec<Uuid>, db: State<'_, DbPool>) -> CmdResult<()> {
    let db = db.inner();

    CommandModel::delete_many_logs(db, &log_ids).await?;

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
    let command = CommandModel::get_by_id(db, command_id)
        .await?
        .context("command not found")?;
    let executions = command.get_executions(db, query).await?;

    Ok(executions)
}

#[tauri::command]
pub async fn delete_command_executions(
    execution_ids: Vec<Uuid>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();

    CommandModel::delete_many_executions(db, &execution_ids).await?;

    Ok(())
}
