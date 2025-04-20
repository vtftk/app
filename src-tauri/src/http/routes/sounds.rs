use anyhow::Context;
use axum::{Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{entity::sounds::SoundModel, DbPool},
    http::error::HttpResult,
};

/// POST /items/all
///
/// List all sounds
pub async fn all(Extension(db): Extension<DbPool>) -> HttpResult<Vec<SoundModel>> {
    let sounds = SoundModel::all(&db).await.context("failed to get sounds")?;
    Ok(Json(sounds))
}

#[derive(Deserialize)]
pub struct QueryByName {
    names: Vec<String>,
    ignore_case: bool,
}

/// POST /sounds/query-by-name
///
/// Query the list of sounds by name
pub async fn query_by_name(
    Extension(db): Extension<DbPool>,
    Json(req): Json<QueryByName>,
) -> HttpResult<Vec<SoundModel>> {
    let sounds = SoundModel::get_by_names(&db, &req.names, req.ignore_case)
        .await
        .context("failed to get sounds")?;
    Ok(Json(sounds))
}

#[derive(Deserialize)]
pub struct QueryById {
    ids: Vec<Uuid>,
}

/// POST /sounds/query-by-id
///
/// Query the list of sounds by id
pub async fn query_by_id(
    Extension(db): Extension<DbPool>,
    Json(req): Json<QueryById>,
) -> HttpResult<Vec<SoundModel>> {
    let sounds = SoundModel::get_by_ids(&db, &req.ids)
        .await
        .context("failed to get sounds")?;
    Ok(Json(sounds))
}
