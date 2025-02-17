//! # Sounds
//!
//! Commands for interacting with sounds from the frontend

use crate::{
    database::{
        entity::{
            shared::UpdateOrdering,
            sounds::{CreateSound, SoundModel, UpdateSound},
        },
        DbPool,
    },
    storage::Storage,
};
use anyhow::Context;
use tauri::State;
use uuid::Uuid;

use super::CmdResult;

/// Get all sounds
#[tauri::command]
pub async fn get_sounds(db: State<'_, DbPool>) -> CmdResult<Vec<SoundModel>> {
    let db = db.inner();
    let sounds = SoundModel::all(db).await?;
    Ok(sounds)
}

/// Get a specific sound by ID
#[tauri::command]
pub async fn get_sound_by_id(
    sound_id: Uuid,
    db: State<'_, DbPool>,
) -> CmdResult<Option<SoundModel>> {
    let db = db.inner();
    let sound = SoundModel::get_by_id(db, sound_id).await?;
    Ok(sound)
}

/// Create a new sound
#[tauri::command]
pub async fn create_sound(create: CreateSound, db: State<'_, DbPool>) -> CmdResult<SoundModel> {
    let db = db.inner();
    let sound = SoundModel::create(db, create).await?;
    Ok(sound)
}

/// Update an existing sound
#[tauri::command]
pub async fn update_sound(
    sound_id: Uuid,
    update: UpdateSound,
    db: State<'_, DbPool>,
    storage: State<'_, Storage>,
) -> CmdResult<SoundModel> {
    let db = db.inner();
    let mut sound = SoundModel::get_by_id(db, sound_id)
        .await?
        .context("sound not found")?;
    let original_sound_url = sound.src.clone();
    sound.update(db, update).await?;

    // Delete previous sound file when changed
    if sound.src != original_sound_url {
        storage.try_delete_file(original_sound_url).await?;
    }

    Ok(sound)
}

/// Delete a sound
#[tauri::command]
pub async fn delete_sound(
    sound_id: Uuid,
    db: State<'_, DbPool>,
    storage: State<'_, Storage>,
) -> CmdResult<()> {
    let db = db.inner();
    let sound = SoundModel::get_by_id(db, sound_id)
        .await?
        .context("sound not found")?;

    let sound_url = sound.src.clone();

    sound.delete(db).await?;

    storage.try_delete_file(sound_url).await?;

    Ok(())
}

#[tauri::command]
pub async fn update_sound_orderings(
    update: Vec<UpdateOrdering>,
    db: State<'_, DbPool>,
) -> CmdResult<()> {
    let db = db.inner();
    SoundModel::update_order(db, update).await?;

    Ok(())
}
