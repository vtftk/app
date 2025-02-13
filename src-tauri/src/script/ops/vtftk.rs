use std::{cell::RefCell, rc::Rc};

use crate::{
    database::entity::{items::ItemModel, items_sounds::SoundType, sounds::SoundModel},
    events::{OverlayMessage, ItemWithSoundIds},
    script::runtime::ScriptRuntimeData,
};
use anyhow::Context;
use deno_core::{op2, OpState};
use uuid::Uuid;

/// Emit event messages to the websocket
#[op2(async)]
#[serde]
pub async fn op_vtftk_emit_event_message(
    state: Rc<RefCell<OpState>>,
    #[serde] message: OverlayMessage,
) -> anyhow::Result<()> {
    let event_sender = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.event_sender.clone()
    };

    event_sender
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
) -> anyhow::Result<Vec<ItemWithSoundIds>> {
    let db = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.db.clone()
    };

    let items: Vec<ItemWithSoundIds> =
        ItemModel::get_by_names_with_sounds(&db, &names, ignore_case)
            .await?
            .into_iter()
            .map(|(item, sounds)| {
                let mut impact_sound_ids = Vec::new();
                let mut windup_sound_ids = Vec::new();

                for sound in sounds {
                    match sound.sound_type {
                        SoundType::Impact => impact_sound_ids.push(sound.sound_id),
                        SoundType::Windup => windup_sound_ids.push(sound.sound_id),
                    }
                }

                ItemWithSoundIds {
                    item,
                    impact_sound_ids,
                    windup_sound_ids,
                }
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
) -> anyhow::Result<Vec<ItemWithSoundIds>> {
    let db = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.db.clone()
    };

    let items: Vec<ItemWithSoundIds> = ItemModel::get_by_ids_with_sounds(&db, &ids)
        .await?
        .into_iter()
        .map(|(item, sounds)| {
            let mut impact_sound_ids = Vec::new();
            let mut windup_sound_ids = Vec::new();

            for sound in sounds {
                match sound.sound_type {
                    SoundType::Impact => impact_sound_ids.push(sound.sound_id),
                    SoundType::Windup => windup_sound_ids.push(sound.sound_id),
                }
            }

            ItemWithSoundIds {
                item,
                impact_sound_ids,
                windup_sound_ids,
            }
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
    let db = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.db.clone()
    };

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
    let db = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.db.clone()
    };
    let sounds = SoundModel::get_by_ids(&db, &ids).await?;
    Ok(sounds)
}
