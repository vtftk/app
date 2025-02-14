use crate::{
    database::entity::{
        chat_history::ChatHistoryModel,
        events::{EventModel, EventTrigger, EventTriggerType},
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

pub struct ScheduledEvent {
    pub event: EventModel,
    /// Next instance the
    pub next_run: Instant,
}

impl Eq for ScheduledEvent {}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.id.eq(&other.event.id)
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
pub struct SchedulerHandle(mpsc::Sender<Vec<EventModel>>);

impl SchedulerHandle {
    pub async fn update_events(&self, events: Vec<EventModel>) -> anyhow::Result<()> {
        self.0.send(events).await.context("failed to send event")
    }
}

pub fn create_scheduler(
    db: DatabaseConnection,
    twitch: Twitch,
    script_handle: ScriptExecutorHandle,
    event_sender: OverlayMessageSender,
) -> SchedulerHandle {
    let (tx, rx) = mpsc::channel(5);

    // Load the initial events data
    tauri::async_runtime::spawn({
        let db = db.clone();
        let tx = tx.clone();

        async move {
            let events = match EventModel::get_by_trigger_type(&db, EventTriggerType::Timer).await {
                Ok(events) => events,
                _ => return,
            };

            _ = tx.send(events).await;
        }
    });

    tauri::async_runtime::spawn(SchedulerEventLoop {
        rx,
        events: BinaryHeap::new(),
        current_sleep: None,
        db,
        twitch,
        script_handle,
        event_sender,
    });

    SchedulerHandle(tx)
}

struct SchedulerEventLoop {
    /// Receiver for the latest events list
    rx: mpsc::Receiver<Vec<EventModel>>,

    /// Heap of scheduled events, ordered by the event which is
    /// due to come first
    events: BinaryHeap<ScheduledEvent>,

    /// Current sleep future
    current_sleep: Option<BoxFuture<'static, ()>>,

    db: DatabaseConnection,
    twitch: Twitch,
    script_handle: ScriptExecutorHandle,
    event_sender: OverlayMessageSender,
}

async fn execute_scheduled_event(
    db: DatabaseConnection,
    twitch: Twitch,
    script_handle: ScriptExecutorHandle,
    event_sender: OverlayMessageSender,
    event: EventModel,
) -> anyhow::Result<()> {
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

    let user_id = twitch.get_user_id().await;

    // Ensure minimum chat messages has been reached
    if min_chat_messages > 0 {
        let last_execution = event
            .last_execution(&db, 0)
            .await
            .context("failed to get last execution")?;

        if let Some(last_execution) = last_execution {
            let message_count =
                ChatHistoryModel::count_since(&db, last_execution.created_at, user_id).await?;

            if message_count < min_chat_messages as u64 {
                debug!("skipping timer execution, not enough chat messages since last execution");
                return Ok(());
            }
        }
    }

    execute_event(
        &db,
        &twitch,
        &script_handle,
        &event_sender,
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
    fn poll_inner(&mut self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        // Accept messages to update the events list
        while let Poll::Ready(Some(events)) = self.rx.poll_recv(cx) {
            // Create the scheduled events
            self.events = events
                .into_iter()
                .filter_map(create_scheduled_event)
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
            tauri::async_runtime::spawn({
                let event = event.event.clone();
                let db = self.db.clone();
                let twitch = self.twitch.clone();
                let script_handle = self.script_handle.clone();
                let event_sender = self.event_sender.clone();

                async move {
                    if let Err(err) =
                        execute_scheduled_event(db, twitch, script_handle, event_sender, event)
                            .await
                    {
                        error!("error while executing event outcome (in timer): {err:?}");
                    }
                }
            });

            if let Some(event) = create_scheduled_event(event.event) {
                self.events.push(event);
            }

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

fn create_scheduled_event(event: EventModel) -> Option<ScheduledEvent> {
    let interval = match &event.trigger {
        EventTrigger::Timer { interval, .. } => *interval,
        _ => return None,
    };

    let next_run = get_next_interval_instant(interval);
    Some(ScheduledEvent { event, next_run })
}

/// Gets the next instant for a fixed interval
fn get_next_interval_instant(interval: u64) -> Instant {
    let now = Local::now();
    let seconds_since_epoch = now.timestamp() as u64;
    let next = (seconds_since_epoch / interval + 1) * interval;
    Instant::now() + Duration::from_secs(next - seconds_since_epoch)
}
