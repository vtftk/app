use serde::{Deserialize, Deserializer, Serialize};

use crate::overlay::VTubeStudioHotkey;

pub mod calibration;

#[derive(Debug, Deserialize)]
pub struct SetAuthTokenRequest {
    pub auth_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetAuthTokenResponse {
    pub auth_token: Option<String>,
}

/// Partial update to the runtime app data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UpdateRuntimeAppData {
    #[serde(default, deserialize_with = "deserialize_some")]
    pub model_id: Option<Option<String>>,
    pub vtube_studio_connected: Option<bool>,
    pub vtube_studio_auth: Option<bool>,
    pub hotkeys: Option<Vec<VTubeStudioHotkey>>,
}

// Any value that is present is considered Some value, including null.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
