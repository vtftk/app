use crate::{
    commands::CmdResult,
    database::entity::model_data::ModelDataModel,
    events::{EventMessageChannel, OverlayMessage},
    http::models::calibration::CalibrationStep,
};
use sea_orm::DatabaseConnection;
use tauri::State;

/// Set the current calibration step
#[tauri::command]
pub fn set_calibration_step(
    step: CalibrationStep,
    event_sender: State<'_, EventMessageChannel>,
) -> CmdResult<()> {
    event_sender.send(OverlayMessage::SetCalibrationStep { step })?;
    Ok(())
}

/// Moves the VTube Studio model by the provided relative amount
#[tauri::command]
pub fn calibration_move_model(
    x: f32,
    y: f32,
    event_sender: State<'_, EventMessageChannel>,
) -> CmdResult<()> {
    event_sender.send(OverlayMessage::MoveModel { x, y })?;
    Ok(())
}

/// Obtains the calibration data for all models
#[tauri::command]
pub async fn get_calibration_data(
    db: State<'_, DatabaseConnection>,
) -> CmdResult<Vec<ModelDataModel>> {
    let db = db.inner();
    let model_data = ModelDataModel::all(db).await?;
    Ok(model_data)
}
