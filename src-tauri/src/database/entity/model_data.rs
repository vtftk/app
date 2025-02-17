use crate::database::{DbPool, DbResult};
use sea_query::{IdenStatic, OnConflict, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub async fn create(db: &DbPool, create: CreateModelData) -> anyhow::Result<ModelDataModel> {
        let model = ModelDataModel {
            id: create.id,
            name: create.name,
            calibration: create.calibration,
        };

        let calibration_value = serde_json::to_value(&model.calibration)?;
        let (sql, values) = Query::insert()
            .into_table(ModelDataTable)
            .columns(ModelDataModel::columns())
            .values_panic([
                model.id.clone().into(),
                model.name.clone().into(),
                calibration_value.into(),
            ])
            .on_conflict(
                OnConflict::new()
                    .update_column(ModelDataColumn::Name)
                    .update_column(ModelDataColumn::Calibration)
                    .to_owned(),
            )
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(model)
    }

    /// Find all model data
    pub async fn all(db: &DbPool) -> DbResult<Vec<ModelDataModel>> {
        let (sql, values) = Query::select()
            .columns(ModelDataModel::columns())
            .from(ModelDataTable)
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(result)
    }

    pub fn columns() -> [ModelDataColumn; 3] {
        [
            ModelDataColumn::Id,
            ModelDataColumn::Name,
            ModelDataColumn::Calibration,
        ]
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "model_data")]
pub struct ModelDataTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ModelDataColumn {
    Id,
    Name,
    Calibration,
}
