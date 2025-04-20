use anyhow::Context;
use axum::{Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        entity::items::{ItemModel, ItemWithSounds},
        DbPool,
    },
    http::error::HttpResult,
};

/// POST /items/all
///
/// List all items
pub async fn all(Extension(db): Extension<DbPool>) -> HttpResult<Vec<ItemModel>> {
    let items = ItemModel::all(&db).await.context("failed to get items")?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct QueryByName {
    names: Vec<String>,
    ignore_case: bool,
}

/// POST /items/query-by-name
///
/// Query the list of items by name
pub async fn query_by_name(
    Extension(db): Extension<DbPool>,
    Json(req): Json<QueryByName>,
) -> HttpResult<Vec<ItemWithSounds>> {
    let items = ItemModel::get_by_names_with_sounds(&db, &req.names, req.ignore_case)
        .await
        .context("failed to get items")?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct QueryById {
    ids: Vec<Uuid>,
}

/// POST /items/query-by-id
///
/// Query the list of items by id
pub async fn query_by_id(
    Extension(db): Extension<DbPool>,
    Json(req): Json<QueryById>,
) -> HttpResult<Vec<ItemWithSounds>> {
    let items = ItemModel::get_by_ids_with_sounds(&db, &req.ids)
        .await
        .context("failed to get items")?;
    Ok(Json(items))
}
