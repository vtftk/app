use crate::{
    database::entity::{app_data::OverlayConfig, items::ItemConfig, sounds::PartialSoundModel},
    http::models::CalibrationStep,
};
use axum::response::sse::Event;
use futures::Stream;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    pin::Pin,
    task::{ready, Poll},
};
use std::{fmt::Debug, sync::Arc};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

/// Embedded overlay HTML browser page
pub const OVERLAY_PAGE: &str = include_str!("../../overlay/dist/index.html");

/// Collection of items along with the resolved impact
/// sounds for the items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsWithSounds {
    /// All the referenced items
    pub items: Vec<PartialItemModel>,
    /// All the referenced sounds
    pub sounds: Vec<PartialSoundModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialItemModel {
    /// Unique ID for the item
    pub id: Uuid,
    /// Image to use for the throwable item
    pub config: ItemConfig,
    /// IDs of impact sounds used by this item
    pub impact_sound_ids: Vec<Uuid>,
    /// IDs of windup sounds used by this item
    pub windup_sound_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThrowItemConfig {
    /// Throw all items at once
    All { amount: i64 },
    /// Throw items in a barrage at a specific frequency
    Barrage {
        amount_per_throw: u32,
        amount: i64,
        frequency: u32,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OverlayMessage {
    // Sets the current calibration step
    SetCalibrationStep {
        step: CalibrationStep,
    },

    // Move the model for calibration purposes
    MoveModel {
        x: f32,
        y: f32,
    },

    /// Throw item
    ThrowItem {
        /// Items to throw
        items: ItemsWithSounds,
        /// Type of throw
        config: ThrowItemConfig,
    },

    /// Request the latest set of vtube studio hotkeys
    UpdateHotkeys,

    /// Trigger a vtube studio hotkey
    TriggerHotkey {
        hotkey_id: String,
    },

    /// Trigger a vtube studio hotkey by name
    TriggerHotkeyByName {
        hotkey_name: String,
        ignore_case: bool,
    },

    /// Play a sound
    PlaySound {
        config: PartialSoundModel,
    },

    /// Play a sequence of sounds one after the other
    PlaySoundSeq {
        configs: Vec<PartialSoundModel>,
    },

    /// Tell the overlay to reload the app data as it
    /// has changed
    ConfigUpdated {
        config: Box<OverlayConfig>,
    },
}

pub struct OverlayMessageReceiver(pub broadcast::Receiver<OverlayMessage>);

impl Clone for OverlayMessageReceiver {
    fn clone(&self) -> Self {
        Self(self.0.resubscribe())
    }
}

pub type OverlayMessageSender = broadcast::Sender<OverlayMessage>;

pub fn create_overlay_channel() -> (OverlayMessageSender, OverlayMessageReceiver) {
    let (tx, rx) = broadcast::channel(10);
    let rx = OverlayMessageReceiver(rx);

    (tx, rx)
}

/// Stream for emitting events to overlays
pub struct OverlayEventStream {
    // Stream of overlay events
    stream: BroadcastStream<OverlayMessage>,
    // Guard held to keep the overlay count active
    _guard: OverlayGuard,
}

impl OverlayEventStream {
    pub fn new(guard: OverlayGuard, overlay_msg_rx: OverlayMessageReceiver) -> Self {
        let stream = BroadcastStream::new(overlay_msg_rx.0);

        OverlayEventStream {
            _guard: guard,
            stream,
        }
    }
}

impl Stream for OverlayEventStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);
        let event = match ready!(stream.poll_next(cx)) {
            Some(Ok(value)) => value,
            _ => return Poll::Ready(None),
        };

        let event = match Event::default().json_data(event) {
            Ok(value) => value,
            _ => return Poll::Ready(None),
        };

        Poll::Ready(Some(Ok(event)))
    }
}

/// Store for [OverlayData] when the state changes the client frontend
/// receives an event containing the new data
#[derive(Clone)]
pub struct OverlayDataStore {
    inner: Arc<OverlayDataStoreInner>,
}

/// Guard held by a runtime app data store indicating that
/// an overlay is active, decrease the reference count of
/// overlays connected when dropped
pub struct OverlayGuard {
    inner: OverlayDataStore,
}

impl Drop for OverlayGuard {
    fn drop(&mut self) {
        // Decrease the counter of active streams
        self.inner.write(|app_data| {
            app_data.active_overlay_count = app_data.active_overlay_count.saturating_sub(1);

            // No longer connected to vtube studio or model
            if app_data.active_overlay_count == 0 {
                app_data.vtube_studio_connected = false;
                app_data.vtube_studio_auth = false;
                app_data.model_id = None;
            }
        });
    }
}

pub struct OverlayDataStoreInner {
    /// Actual current runtime app data
    data: RwLock<OverlayData>,
    /// App handle to report changes to
    app_handle: AppHandle,
}

impl OverlayDataStore {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            inner: Arc::new(OverlayDataStoreInner {
                data: Default::default(),
                app_handle,
            }),
        }
    }

    /// Get the current overlay data
    pub fn get(&self) -> OverlayData {
        self.inner.data.read().clone()
    }

    /// Creates a new overlay guard
    pub fn create_overlay(&self) -> OverlayGuard {
        // Increase number of active overlays
        self.write(|app_data| {
            app_data.active_overlay_count = app_data.active_overlay_count.saturating_add(1);
        });

        OverlayGuard {
            inner: self.clone(),
        }
    }

    pub fn write<F>(&self, action: F)
    where
        F: FnOnce(&mut OverlayData),
    {
        let data = &mut *self.inner.data.write();
        action(data);

        // Let the frontend know the runtime data has changed
        _ = self
            .inner
            .app_handle
            .emit("runtime_app_data_changed", &data);
    }
}

/// App data used at runtime, used by the overlay for informing the client
/// the current state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OverlayData {
    /// ID of current model
    pub model_id: Option<String>,

    /// vtube studio connection state
    pub vtube_studio_connected: bool,

    /// VTube studio authentication state
    pub vtube_studio_auth: bool,

    /// Current hotkey list from vtube studio
    pub hotkeys: Vec<VTubeStudioHotkey>,

    /// Current number of active connected overlays
    pub active_overlay_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTubeStudioHotkey {
    pub hotkey_id: String,
    pub name: String,
}
