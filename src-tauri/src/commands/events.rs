//! # Events
//!
//! Commands for interacting with events from the frontend

use super::CmdResult;
use crate::{
    database::entity::{
        event_executions::EventExecutionModel,
        event_logs::EventLogsModel,
        events::{CreateEvent, EventModel, EventTrigger, EventTriggerType, UpdateEvent},
        shared::{ExecutionsQuery, LogsQuery, UpdateOrdering},
    },
    events::{
        matching::EventData, outcome::produce_outcome_message, scheduler::SchedulerHandle,
        EventMessageChannel,
    },
    script::runtime::ScriptExecutorHandle,
    twitch::manager::Twitch,
};
use anyhow::Context;
use sea_orm::{DatabaseConnection, ModelTrait};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_events(db: State<'_, DatabaseConnection>) -> CmdResult<Vec<EventModel>> {
    let db = db.inner();
    let events = EventModel::all(db).await?;
    Ok(events)
}

#[tauri::command]
pub async fn get_event_by_id(
    event_id: Uuid,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<Option<EventModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id).await?;
    Ok(event)
}

#[tauri::command]
pub async fn create_event(
    create: CreateEvent,
    db: State<'_, DatabaseConnection>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<EventModel> {
    let db = db.inner();
    let event = EventModel::create(db, create).await?;

    // Update the event scheduler
    if let EventTrigger::Timer { .. } = event.trigger {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(event)
}

#[tauri::command]
pub async fn update_event(
    event_id: Uuid,
    update: UpdateEvent,
    db: State<'_, DatabaseConnection>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<EventModel> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;
    let event = event.update(db, update).await?;

    // Update the event scheduler
    if let EventTrigger::Timer { .. } = event.trigger {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(event)
}

#[tauri::command]
pub async fn delete_event(
    event_id: Uuid,
    db: State<'_, DatabaseConnection>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<()> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;

    let is_timer_event = matches!(event.trigger, EventTrigger::Timer { .. });

    event.delete(db).await?;

    // Update the event scheduler to handle deleted timer
    if is_timer_event {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(())
}

async fn update_scheduler_events(db: &DatabaseConnection, scheduler: &SchedulerHandle) {
    if let Ok(events) = EventModel::get_by_trigger_type(db, EventTriggerType::Timer).await {
        _ = scheduler.update_events(events).await;
    }
}

#[tauri::command]
pub async fn test_event_by_id(
    event_id: Uuid,
    event_data: EventData,
    db: State<'_, DatabaseConnection>,
    event_sender: State<'_, EventMessageChannel>,
    twitch: State<'_, Twitch>,
    script_handle: State<'_, ScriptExecutorHandle>,
) -> CmdResult<()> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("unknown event")?;

    if let Some(msg) =
        produce_outcome_message(db, &twitch, &script_handle, event, event_data).await?
    {
        _ = event_sender.send(msg);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_event_orderings(
    update: Vec<UpdateOrdering>,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<()> {
    let db = db.inner();
    EventModel::update_order(db, update).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_event_executions(
    event_id: Uuid,
    query: ExecutionsQuery,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<Vec<EventExecutionModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("unknown event")?;
    let executions = event.get_executions(db, query).await?;
    Ok(executions)
}

#[tauri::command]
pub async fn delete_event_executions(
    execution_ids: Vec<Uuid>,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<()> {
    let db = db.inner();
    EventExecutionModel::delete_many(db, &execution_ids).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_event_logs(
    event_id: Uuid,
    query: LogsQuery,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<Vec<EventLogsModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;
    let logs = event.get_logs(db, query).await?;
    Ok(logs)
}

#[tauri::command]
pub async fn delete_event_logs(
    log_ids: Vec<Uuid>,
    db: State<'_, DatabaseConnection>,
) -> CmdResult<()> {
    let db = db.inner();
    EventLogsModel::delete_many(db, &log_ids).await?;
    Ok(())
}
