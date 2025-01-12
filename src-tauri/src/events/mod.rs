pub mod matching;
pub mod outcome;
pub mod processing;
pub mod scheduler;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    database::entity::{app_data::AppData, ItemModel, SoundModel},
    http::models::calibration::CalibrationStep,
};

/// Collection of items along with the resolved impact
/// sounds for the items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsWithSounds {
    /// All the referenced items
    pub items: Vec<ItemWithImpactSoundIds>,
    /// All the referenced sounds
    pub impact_sounds: Vec<SoundModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemWithImpactSoundIds {
    #[serde(flatten)]
    pub item: ItemModel,
    pub impact_sound_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThrowItemMessage {
    /// Items to throw
    pub items: ItemsWithSounds,
    /// Type of throw
    pub config: ThrowItemConfig,
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

pub type EventMessageChannel = broadcast::Sender<EventMessage>;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum EventMessage {
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
    ThrowItem(ThrowItemMessage),

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
        config: SoundModel,
    },

    /// Play a sequence of sounds one after the other
    PlaySoundSeq {
        configs: Vec<SoundModel>,
    },

    /// Tell the overlay to reload the app data as it
    /// has changed
    AppDataUpdated {
        app_data: Box<AppData>,
    },
}

pub struct EventRecvHandle(pub broadcast::Receiver<EventMessage>);

impl Clone for EventRecvHandle {
    fn clone(&self) -> Self {
        Self(self.0.resubscribe())
    }
}

pub type EventSendHandle = broadcast::Sender<EventMessage>;

pub fn create_event_channel() -> (EventSendHandle, EventRecvHandle) {
    let (event_tx, rx) = broadcast::channel(10);
    let event_rx = EventRecvHandle(rx);

    (event_tx, event_rx)
}
