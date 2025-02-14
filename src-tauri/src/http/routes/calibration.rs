use anyhow::Context;
use axum::{Extension, Json};
use log::info;
use sea_orm::DatabaseConnection;
use tauri::{AppHandle, Emitter};

use crate::{
    database::entity::{
        model_data::{CreateModelData, ModelCalibration, ModelDataModel},
        shared::MinMax,
    },
    http::{
        error::HttpResult,
        models::calibration::{CalibrationProgressRes, CalibrationStepData},
    },
};

/// GET /calibration
///
/// Requests the current calibration data for all models
pub async fn handle_calibration_data(
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<Vec<ModelDataModel>> {
    Ok(Json(
        ModelDataModel::all(&db)
            .await
            .context("failed to load calibration data")?,
    ))
}

/// POST /calibration
///
/// Handles updating the calibration progress
pub async fn handle_calibration_progress(
    Extension(db): Extension<DatabaseConnection>,
    Extension(app_handle): Extension<AppHandle>,
    Json(req): Json<CalibrationStepData>,
) -> HttpResult<CalibrationProgressRes> {
    // Handle completed calibration
    let model_data = if let CalibrationStepData::Complete {
        model_id,
        model_name,
        smallest_point,
        largest_point,
    } = &req
    {
        info!(
            "COMPLETED CALIBRATION: {:?} {:?}",
            smallest_point, largest_point
        );

        let model = ModelDataModel::create(
            &db,
            CreateModelData {
                id: model_id.clone(),
                name: model_name.clone(),
                calibration: ModelCalibration {
                    x: MinMax {
                        min: smallest_point.x,
                        max: largest_point.x,
                    },
                    y: MinMax {
                        min: smallest_point.y,
                        max: largest_point.y,
                    },
                },
            },
        )
        .await
        .context("create model data")?;

        app_handle
            .emit("model_data_updated", &model)
            .context("failed to inform app of model data update")?;
        Some(model)
    } else {
        None
    };

    app_handle
        .emit("calibration_state", req)
        .context("failed to inform app")?;

    Ok(Json(CalibrationProgressRes { model_data }))
}
