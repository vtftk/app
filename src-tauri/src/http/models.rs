use crate::{
    database::entity::model_data::{ModelDataModel, ModelId},
    overlay::VTubeStudioHotkey,
};
use serde::{Deserialize, Deserializer, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "step")]
pub struct CalibrationPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "step")]
pub enum CalibrationStepData {
    NotStarted,
    Smallest,
    Largest,
    Complete {
        model_id: ModelId,
        model_name: String,
        smallest_point: CalibrationPoint,
        largest_point: CalibrationPoint,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CalibrationStep {
    NotStarted,
    Smallest,
    Largest,
    Complete,
}

#[derive(Debug, Serialize)]
pub struct CalibrationProgressRes {
    /// Updated model data when a model calibration is complete
    pub model_data: Option<ModelDataModel>,
}
