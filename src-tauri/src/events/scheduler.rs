use anyhow::Context;
use chrono::Local;
use futures::future::BoxFuture;
use log::error;
use std::{collections::BinaryHeap, future::Future, pin::Pin, task::Poll, time::Duration};
use tokio::{
    sync::mpsc,
    time::{sleep_until, Instant},
};
use uuid::Uuid;

use super::{AppEvent, AppEventSender, TimerCompleted};

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

pub fn create_scheduler(event_tx: AppEventSender) -> SchedulerHandle {
    let (tx, rx) = mpsc::channel(5);
    let handle = SchedulerHandle(tx);

    tauri::async_runtime::spawn(SchedulerEventLoop {
        rx,
        events: BinaryHeap::new(),
        current_sleep: None,
        event_tx,
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

    event_tx: AppEventSender,
}

impl SchedulerEventLoop {
    fn execute_event(event_id: Uuid, event_tx: AppEventSender) {
        if let Err(err) = event_tx.send(AppEvent::TimerCompleted(TimerCompleted { event_id })) {
            error!("failed to send timer complete event, event loop stopped: {err:?}");
        }
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
            Self::execute_event(event.event_id, self.event_tx.clone());

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
