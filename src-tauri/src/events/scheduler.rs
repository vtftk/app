use crate::{
    database::entity::{
        chat_history::ChatHistoryModel,
        events::{EventModel, EventTrigger},
    },
    events::{
        matching::{EventData, EventInputData},
        processing::execute_event,
    },
    overlay::OverlayMessageSender,
    script::runtime::ScriptExecutorHandle,
    twitch::manager::Twitch,
};
use anyhow::Context;
use chrono::Local;
use futures::future::BoxFuture;
use log::{debug, error};
use sea_orm::DatabaseConnection;
use std::{collections::BinaryHeap, future::Future, pin::Pin, task::Poll, time::Duration};
use tokio::{
    sync::mpsc,
    time::{sleep_until, Instant},
};
use uuid::Uuid;

pub struct ScheduledEvent {
    /// ID of the event to execute
    pub event_id: Uuid,

    /// Interval the event executes at
    /// (For further scheduling)
    pub interval: u64,

    /// Next instance the
    pub next_run: Instant,
}

impl Eq for ScheduledEvent {}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event_id.eq(&other.event_id)
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse comparison order for binary heap to sort
        // closest ones to the top
        other.next_run.cmp(&self.next_run)
    }
}

#[derive(Clone)]
pub struct SchedulerHandle(mpsc::Sender<Vec<SchedulerQueueEvent>>);

pub struct SchedulerQueueEvent {
    /// ID of the event to execute
    pub event_id: Uuid,
    /// Interval the event executes at
    /// (For further scheduling)
    pub interval: u64,
}

impl SchedulerHandle {
    pub async fn update_events(&self, events: Vec<SchedulerQueueEvent>) -> anyhow::Result<()> {
        self.0.send(events).await.context("failed to send event")
    }
}

/// Context for executing events
#[derive(Clone)]
pub struct SchedulerContext {
    db: DatabaseConnection,
    twitch: Twitch,
    script_handle: ScriptExecutorHandle,
    overlay_tx: OverlayMessageSender,
}

impl SchedulerContext {
    pub fn new(
        db: DatabaseConnection,
        twitch: Twitch,
        script_handle: ScriptExecutorHandle,
        overlay_tx: OverlayMessageSender,
    ) -> Self {
        Self {
            db,
            twitch,
            script_handle,
            overlay_tx,
        }
    }
}

pub fn create_scheduler(ctx: SchedulerContext) -> SchedulerHandle {
    let (tx, rx) = mpsc::channel(5);
    let handle = SchedulerHandle(tx);

    tauri::async_runtime::spawn(SchedulerEventLoop {
        rx,
        events: BinaryHeap::new(),
        current_sleep: None,
        ctx,
    });

    handle
}

struct SchedulerEventLoop {
    /// Receiver for the latest events list
    rx: mpsc::Receiver<Vec<SchedulerQueueEvent>>,

    /// Heap of scheduled events, ordered by the event which is
    /// due to come first
    events: BinaryHeap<ScheduledEvent>,

    /// Current sleep future
    current_sleep: Option<BoxFuture<'static, ()>>,

    ctx: SchedulerContext,
}

async fn execute_scheduled_event(event_id: Uuid, ctx: SchedulerContext) -> anyhow::Result<()> {
    let db = &ctx.db;

    let event = EventModel::get_by_id(db, event_id)
        .await
        .context("failed to get event to trigger")?
        .context("unknown event")?;

    let min_chat_messages = match &event.trigger {
        EventTrigger::Timer {
            min_chat_messages, ..
        } => *min_chat_messages,
        _ => {
            return Err(anyhow::anyhow!(
                "attempted to execute timer event that was not a timer event"
            ));
        }
    };

    let user_id = ctx.twitch.get_user_id().await;

    // Ensure minimum chat messages has been reached
    if min_chat_messages > 0 {
        let last_execution = event
            .last_execution(&ctx.db, 0)
            .await
            .context("failed to get last execution")?;

        if let Some(last_execution) = last_execution {
            let message_count =
                ChatHistoryModel::count_since(&ctx.db, last_execution.created_at, user_id).await?;

            if message_count < min_chat_messages as u64 {
                debug!("skipping timer execution, not enough chat messages since last execution");
                return Ok(());
            }
        }
    }

    execute_event(
        &ctx.db,
        &ctx.twitch,
        &ctx.script_handle,
        &ctx.overlay_tx,
        event,
        EventData {
            user: None,
            input_data: EventInputData::None,
        },
    )
    .await?;

    Ok(())
}

impl SchedulerEventLoop {
    fn execute_event(event_id: Uuid, ctx: SchedulerContext) {
        // Trigger the event
        tauri::async_runtime::spawn({
            async move {
                if let Err(err) = execute_scheduled_event(event_id, ctx).await {
                    error!("error while executing event outcome (in timer): {err:?}");
                }
            }
        });
    }

    fn poll_inner(&mut self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        // Accept messages to update the events list
        while let Poll::Ready(Some(events)) = self.rx.poll_recv(cx) {
            // Create the scheduled events
            self.events = events
                .into_iter()
                .map(|event| create_scheduled_event(event.event_id, event.interval))
                .collect();

            // Clear sleep state
            self.current_sleep = None;
        }

        if let Some(current_sleep) = self.current_sleep.as_mut() {
            // Poll current sleep
            if Pin::new(current_sleep).poll(cx).is_pending() {
                return Poll::Pending;
            }

            // Clear current sleep
            self.current_sleep = None;

            // Value should always be present when we have awaited a sleep state
            let event = match self.events.pop() {
                Some(value) => value,
                None => return Poll::Pending,
            };

            // Trigger the event
            Self::execute_event(event.event_id, self.ctx.clone());

            // Create the next iteration of the event
            self.events
                .push(create_scheduled_event(event.event_id, event.interval));

            // Emit event
            return Poll::Ready(());
        }

        // Peek the top event
        let next_event = match self.events.peek() {
            Some(value) => value,
            None => return Poll::Pending,
        };

        // Store and poll new sleep state
        let sleep = sleep_until(next_event.next_run);
        let sleep = self.current_sleep.insert(Box::pin(sleep));

        Pin::new(sleep).poll(cx)
    }
}

impl Future for SchedulerEventLoop {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        // Poll inner until its no longer ready
        while this.poll_inner(cx).is_ready() {}

        Poll::Pending
    }
}

fn create_scheduled_event(event_id: Uuid, interval: u64) -> ScheduledEvent {
    let next_run = get_next_interval_instant(interval);
    ScheduledEvent {
        event_id,
        interval,
        next_run,
    }
}

/// Gets the next instant for a fixed interval
fn get_next_interval_instant(interval: u64) -> Instant {
    let now = Local::now();
    let seconds_since_epoch = now.timestamp() as u64;
    let next = (seconds_since_epoch / interval + 1) * interval;
    Instant::now() + Duration::from_secs(next - seconds_since_epoch)
}
