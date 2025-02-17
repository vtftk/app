//! # Items
//!
//! Commands for interacting with items from the frontend

use super::CmdResult;
use crate::{
    database::{
        entity::{
            items::{CreateItem, ItemModel, ItemWithSounds, UpdateItem},
            shared::UpdateOrdering,
            sounds::SoundType,
        },
        DbPool,
    },
    storage::Storage,
};
use anyhow::Context;
use tauri::State;
use uuid::Uuid;

/// Get all items
#[tauri::command]
pub async fn get_items(db: State<'_, DbPool>) -> CmdResult<Vec<ItemModel>> {
    let db = db.inner();
    let items = ItemModel::all(db).await?;
    Ok(items)
}

/// Get a specific item by ID, provides both the item itself
/// and any associated impact sounds
#[tauri::command]
pub async fn get_item_by_id(
    item_id: Uuid,
    db: State<'_, DbPool>,
) -> CmdResult<Option<ItemWithSounds>> {
    let db = db.inner();
    let item = match ItemModel::get_by_id(db, item_id).await? {
        Some(value) => value,
        None => return Ok(None),
    };

    let item_with_sounds = item.with_sounds(db).await?;

    Ok(Some(item_with_sounds))
}

/// Create a new item
#[tauri::command]
pub async fn create_item(create: CreateItem, db: State<'_, DbPool>) -> CmdResult<ItemWithSounds> {
    let db = db.inner();
    let item = ItemModel::create(db, create).await?;
    let item_with_sounds = item.with_sounds(db).await?;

    Ok(item_with_sounds)
}

/// Update an existing item
#[tauri::command]
pub async fn update_item(
    item_id: Uuid,
    update: UpdateItem,
    db: State<'_, DbPool>,
    storage: State<'_, Storage>,
) -> CmdResult<ItemWithSounds> {
    let db = db.inner();
    let mut item = ItemModel::get_by_id(db, item_id)
        .await?
        .context("item not found")?;

    let original_item_url = item.config.image.src.clone();

    item.update(db, update).await?;

    // Delete previous image file when changed
    if item.config.image.src != original_item_url {
        storage.try_delete_file(original_item_url).await?;
    }

    let item_with_sounds = item.with_sounds(db).await?;
    Ok(item_with_sounds)
}

/// Updates the list orderings of items using the provided orderings
#[tauri::command]
pub async fn update_item_orderings(
    update: Vec<UpdateOrdering>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    ItemModel::update_order(db, update).await?;

    Ok(())
}

/// Add impact sounds to an item
#[tauri::command]
pub async fn append_item_impact_sounds(
    item_id: Uuid,
    sounds: Vec<Uuid>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    let item = ItemModel::get_by_id(db, item_id)
        .await?
        .context("item not found")?;
    item.append_sounds(db, &sounds, SoundType::Impact).await?;
    Ok(())
}

/// Delete an item
#[tauri::command]
pub async fn delete_item(
    item_id: Uuid,
    db: State<'_, DbPool>,
    storage: State<'_, Storage>,
) -> CmdResult<()> {
    let db = db.inner();
    let item = ItemModel::get_by_id(db, item_id)
        .await?
        .context("item not found")?;

    let item_url = item.config.image.src.clone();

    item.delete(db).await?;

    storage.try_delete_file(item_url).await?;

    Ok(())
}
