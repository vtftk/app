//! # Events
//!
//! Commands for interacting with events from the frontend

use super::CmdResult;
use crate::{
    database::{
        entity::{
            event_execution::EventExecutionModel,
            event_log::EventLogsModel,
            events::{CreateEvent, EventModel, EventTrigger, EventTriggerType, UpdateEvent},
            shared::{ExecutionsQuery, LogsQuery, UpdateOrdering},
        },
        DbPool,
    },
    events::{
        matching::EventData,
        outcome::produce_outcome_message,
        scheduler::{SchedulerHandle, SchedulerQueueEvent},
    },
    overlay::OverlayMessageSender,
    script::runtime::ScriptExecutorHandle,
    twitch::manager::Twitch,
};
use anyhow::Context;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_events(db: State<'_, DbPool>) -> CmdResult<Vec<EventModel>> {
    let db = db.inner();
    let events = EventModel::all(db).await?;
    Ok(events)
}

#[tauri::command]
pub async fn get_event_by_id(
    event_id: Uuid,
    db: State<'_, DbPool>,
) -> CmdResult<Option<EventModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id).await?;
    Ok(event)
}

#[tauri::command]
pub async fn create_event(
    create: CreateEvent,
    db: State<'_, DbPool>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<EventModel> {
    let db = db.inner();
    let event = EventModel::create(db, create).await?;

    // Update the event scheduler
    if let EventTrigger::Timer { .. } = event.config.trigger {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(event)
}

#[tauri::command]
pub async fn update_event(
    event_id: Uuid,
    update: UpdateEvent,
    db: State<'_, DbPool>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<EventModel> {
    let db = db.inner();
    let mut event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;
    event.update(db, update).await?;

    // Update the event scheduler
    if let EventTrigger::Timer { .. } = event.config.trigger {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(event)
}

#[tauri::command]
pub async fn delete_event(
    event_id: Uuid,
    db: State<'_, DbPool>,
    scheduler: State<'_, SchedulerHandle>,
) -> CmdResult<()> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;

    let is_timer_event = matches!(event.config.trigger, EventTrigger::Timer { .. });

    event.delete(db).await?;

    // Update the event scheduler to handle deleted timer
    if is_timer_event {
        update_scheduler_events(db, scheduler.inner()).await;
    }

    Ok(())
}

/// Sets the current set of events for the scheduler by fetching the ucrrent list of
/// timer events from the database
pub async fn update_scheduler_events(db: &DbPool, scheduler: &SchedulerHandle) {
    if let Ok(events) = EventModel::get_by_trigger_type(db, EventTriggerType::Timer).await {
        // Map into scheduler events
        let scheduled = events
            .into_iter()
            .filter_map(|event| {
                let interval = match &event.config.trigger {
                    EventTrigger::Timer { interval, .. } => *interval,
                    _ => return None,
                };

                Some(SchedulerQueueEvent {
                    event_id: event.id,
                    interval,
                })
            })
            .collect();

        _ = scheduler.update_events(scheduled).await;
    }
}

#[tauri::command]
pub async fn test_event_by_id(
    event_id: Uuid,
    event_data: EventData,
    db: State<'_, DbPool>,
    event_sender: State<'_, OverlayMessageSender>,
    twitch: State<'_, Twitch>,
    script_handle: State<'_, ScriptExecutorHandle>,
) -> CmdResult<()> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("unknown event")?;

    if let Some(msg) =
        produce_outcome_message(db, &twitch, &script_handle, event, &event_data).await?
    {
        _ = event_sender.send(msg);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_event_orderings(
    update: Vec<UpdateOrdering>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    EventModel::update_order(db, update).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_event_executions(
    event_id: Uuid,
    query: ExecutionsQuery,
    db: State<'_, DbPool>,
) -> CmdResult<Vec<EventExecutionModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("unknown event")?;

    let executions = EventExecutionModel::query(db, event.id, query).await?;
    Ok(executions)
}

#[tauri::command]
pub async fn delete_event_executions(
    execution_ids: Vec<Uuid>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    EventExecutionModel::delete_by_ids(db, &execution_ids).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_event_logs(
    event_id: Uuid,
    query: LogsQuery,
    db: State<'_, DbPool>,
) -> CmdResult<Vec<EventLogsModel>> {
    let db = db.inner();
    let event = EventModel::get_by_id(db, event_id)
        .await?
        .context("event not found")?;
    let logs = EventLogsModel::query(db, event.id, query).await?;
    Ok(logs)
}

#[tauri::command]
pub async fn delete_event_logs(log_ids: Vec<Uuid>, db: State<'_, DbPool>) -> CmdResult<()> {
    let db = db.inner();
    EventLogsModel::delete_by_ids(db, &log_ids).await?;
    Ok(())
}
