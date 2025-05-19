use crate::database::{DbErr, DbPool, DbResult};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::shared::MinMax;

pub type ModelId = String;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelDataModel {
    /// Unique ID for the sound
    pub id: ModelId,
    /// Name of the model in VT studio
    pub name: String,
    /// Calibration data for the model
    #[sqlx(json)]
    pub calibration: ModelCalibration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCalibration {
    /// Min and max X positions of the model
    pub x: MinMax<f64>,
    /// Min and max Y positions of the model
    pub y: MinMax<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelData {
    pub id: String,
    pub name: String,
    pub calibration: ModelCalibration,
}

impl ModelDataModel {
    /// Create a new script
    pub async fn create(db: &DbPool, create: CreateModelData) -> DbResult<ModelDataModel> {
        let model = ModelDataModel {
            id: create.id,
            name: create.name,
            calibration: create.calibration,
        };

        let calibration_value =
            serde_json::to_value(&model.calibration).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(
            r#"
            INSERT INTO "model_data" ("id", "name", "calibration")
            VALUES (?, ?, ?)
            ON CONFLICT(Id) DO UPDATE SET
                "name" = excluded."name",
                "calibration" = excluded."calibration"
        "#,
        )
        .bind(model.id.as_str())
        .bind(model.name.as_str())
        .bind(calibration_value)
        .execute(db)
        .await?;

        Ok(model)
    }

    /// Find all model data
    pub async fn all(db: &DbPool) -> DbResult<Vec<ModelDataModel>> {
        sqlx::query_as(r#"SELECT * FROM "model_data""#)
            .fetch_all(db)
            .await
    }
}
