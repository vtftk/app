use std::collections::HashSet;

use anyhow::{anyhow, Context};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::entity::{
        events::{
            BitsAmount, EventOutcome, EventOutcomeBits, EventOutcomePlaySound,
            EventOutcomeThrowable, EventOutcomeTriggerHotkey, ThrowableData,
        },
        ItemModel, SoundModel,
    },
    state::app_data::{ItemWithImpactSoundIds, ThrowableConfig},
};

use super::{
    matching::{EventData, EventInputData},
    EventMessage,
};

/// Produce a message for an outcome
pub async fn produce_outcome_message(
    db: &DatabaseConnection,
    event_data: EventData,
    outcome: EventOutcome,
) -> anyhow::Result<EventMessage> {
    match outcome {
        EventOutcome::ThrowBits(data) => throw_bits_outcome(db, event_data, data).await,
        EventOutcome::Throwable(data) => throwable_outcome(db, data).await,
        EventOutcome::TriggerHotkey(data) => trigger_hotkey_outcome(data),
        EventOutcome::PlaySound(data) => play_sound_outcome(db, data).await,
    }
}

/// Produce a bits throwing outcome message
async fn throw_bits_outcome(
    db: &DatabaseConnection,
    event_data: EventData,
    data: EventOutcomeBits,
) -> anyhow::Result<EventMessage> {
    let input = match event_data.input_data {
        EventInputData::Bits { bits, .. } => bits as u32,
        _ => {
            return Err(anyhow!(
                "unexpected event input, throw bits requires bit count"
            ))
        }
    };

    let sets = [data._1, data._100, data._1000, data._5000, data._10000];
    let mut bit_index: usize = match input {
        1..=99 => 0,
        100..=999 => 1,
        1000..=4999 => 2,
        5000..=9999 => 3,
        _ => 4,
    };

    let mut bit_icon: Option<Uuid> = None;

    // Go through the bit icons till we find one
    while bit_icon.is_none() {
        bit_icon = sets.get(bit_index).and_then(|value| *value);

        // Move to index before
        match bit_index.checked_sub(1) {
            Some(value) => {
                bit_index = value;
            }
            None => break,
        }
    }

    let bit_icon = bit_icon.context("no bit icon available")?;

    let throwable_config = create_throwable_config(db, &[bit_icon]).await?;

    let amount = match data.amount {
        BitsAmount::Dynamic { max_amount } => input.min(max_amount),
        BitsAmount::Fixed { amount } => amount,
    };

    Ok(EventMessage::ThrowItem {
        config: throwable_config,
        amount,
    })
}

// Produce a throwable message
async fn throwable_outcome(
    db: &DatabaseConnection,
    data: EventOutcomeThrowable,
) -> anyhow::Result<EventMessage> {
    match data.data {
        ThrowableData::Throw {
            throwable_ids,
            amount,
        } => {
            let throwable_config = create_throwable_config(db, &throwable_ids).await?;

            Ok(EventMessage::ThrowItem {
                config: throwable_config,
                amount,
            })
        }
        ThrowableData::Barrage {
            throwable_ids,
            amount_per_throw,
            frequency,
            amount,
        } => {
            let throwable_config = create_throwable_config(db, &throwable_ids).await?;

            Ok(EventMessage::ThrowItemBarrage {
                config: throwable_config,
                amount,
                frequency,
                amount_per_throw,
            })
        }
    }
}

/// Produce a hotkey trigger message
fn trigger_hotkey_outcome(data: EventOutcomeTriggerHotkey) -> anyhow::Result<EventMessage> {
    Ok(EventMessage::TriggerHotkey {
        hotkey_id: data.hotkey_id,
    })
}

/// Produce a sound outcome event message
async fn play_sound_outcome(
    db: &DatabaseConnection,
    data: EventOutcomePlaySound,
) -> anyhow::Result<EventMessage> {
    let config = SoundModel::get_by_id(db, data.sound_id)
        .await?
        .context("sound config not found")?;

    Ok(EventMessage::PlaySound { config })
}

pub async fn create_throwable_config(
    db: &DatabaseConnection,
    item_ids: &[Uuid],
) -> anyhow::Result<ThrowableConfig> {
    let items: Vec<ItemWithImpactSoundIds> = ItemModel::get_by_ids_with_impact_sounds(db, item_ids)
        .await?
        .into_iter()
        .map(|(item, impact_sounds)| ItemWithImpactSoundIds {
            item,
            impact_sound_ids: impact_sounds
                .into_iter()
                .map(|impact_sound| impact_sound.sound_id)
                .collect(),
        })
        .collect();

    // Collect all unique impact sound IDs
    let impact_sound_ids: Vec<Uuid> = items
        .iter()
        .flat_map(|item| item.impact_sound_ids.iter())
        .cloned()
        // Collect to HashSet first for unique IDs
        .collect::<HashSet<Uuid>>()
        .into_iter()
        .collect::<Vec<Uuid>>();

    let impact_sounds = SoundModel::get_by_ids(db, &impact_sound_ids).await?;

    Ok(ThrowableConfig {
        items,
        impact_sounds,
    })
}