use std::{cell::RefCell, rc::Rc};

use crate::{
    database::entity::{items::ItemModel, sounds::SoundModel},
    overlay::{OverlayMessage, PartialItemModel},
    script::runtime::ScriptRuntimeDataExt,
};
use anyhow::Context;
use deno_core::{op2, OpState};
use uuid::Uuid;

/// Emit event messages to the websocket
#[op2(async)]
#[serde]
pub async fn op_vtftk_emit_overlay_message(
    state: Rc<RefCell<OpState>>,
    #[serde] message: OverlayMessage,
) -> anyhow::Result<()> {
    let overlay_sender = state.overlay_sender()?;

    overlay_sender
        .send(message)
        .context("event receiver was closed")?;

    Ok(())
}

/// Find items by name
#[op2(async)]
#[serde]
pub async fn op_vtftk_get_items_by_names(
    state: Rc<RefCell<OpState>>,
    #[serde] names: Vec<String>,
    ignore_case: bool,
) -> anyhow::Result<Vec<PartialItemModel>> {
    let db = state.db()?;
    let items: Vec<PartialItemModel> =
        ItemModel::get_by_names_with_sounds(&db, &names, ignore_case)
            .await?
            .into_iter()
            .map(|item| PartialItemModel {
                id: item.item.id,
                config: item.item.config,
                impact_sound_ids: item.impact_sounds_ids,
                windup_sound_ids: item.windup_sounds_ids,
            })
            .collect();

    Ok(items)
}

/// Find items by ids
#[op2(async)]
#[serde]
pub async fn op_vtftk_get_items_by_ids(
    state: Rc<RefCell<OpState>>,
    #[serde] ids: Vec<Uuid>,
) -> anyhow::Result<Vec<PartialItemModel>> {
    let db = state.db()?;
    let items: Vec<PartialItemModel> = ItemModel::get_by_ids_with_sounds(&db, &ids)
        .await?
        .into_iter()
        .map(|item| PartialItemModel {
            id: item.item.id,
            config: item.item.config,
            impact_sound_ids: item.impact_sounds_ids,
            windup_sound_ids: item.windup_sounds_ids,
        })
        .collect();

    Ok(items)
}

/// Find sounds by name
#[op2(async)]
#[serde]
pub async fn op_vtftk_get_sounds_by_names(
    state: Rc<RefCell<OpState>>,
    #[serde] names: Vec<String>,
    ignore_case: bool,
) -> anyhow::Result<Vec<SoundModel>> {
    let db = state.db()?;
    let sounds = SoundModel::get_by_names(&db, &names, ignore_case).await?;
    Ok(sounds)
}

/// Find sound by ID
#[op2(async)]
#[serde]
pub async fn op_vtftk_get_sounds_by_ids(
    state: Rc<RefCell<OpState>>,
    #[serde] ids: Vec<Uuid>,
) -> anyhow::Result<Vec<SoundModel>> {
    let db = state.db()?;
    let sounds = SoundModel::get_by_ids(&db, &ids).await?;
    Ok(sounds)
}
