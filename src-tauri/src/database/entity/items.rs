use chrono::{DateTime, Utc};
use sea_query::{CaseStatement, Expr, Func, IdenStatic, OnConflict, Order, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_maybe_one},
    DbPool, DbResult,
};

use super::{shared::UpdateOrdering, sounds::SoundType};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ItemModel {
    /// Unique ID for the item
    pub id: Uuid,
    /// Name of the throwable item
    pub name: String,
    /// Image to use for the throwable item
    #[sqlx(json)]
    pub config: ItemConfig,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemConfig {
    pub image: ItemImageConfig,
    #[serde(default)]
    pub windup: ItemWindupConfig,
}

/// Configuration for a throwable image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemImageConfig {
    /// Src URL for the image
    pub src: String,
    /// Weight of impact the image has
    pub weight: f32,
    /// Scale of the image
    pub scale: f32,
    /// Whether to allow pixelation when rendering at a
    /// different scale
    pub pixelate: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemWindupConfig {
    /// Whether a windup is enabled
    pub enabled: bool,
    /// Duration of the windup
    pub duration: u32,
}

/// Data for updating an item
#[derive(Debug, Default, Deserialize)]
pub struct UpdateItem {
    pub name: Option<String>,
    pub config: Option<ItemConfig>,
    pub impact_sounds: Option<Vec<Uuid>>,
    pub windup_sounds: Option<Vec<Uuid>>,
    pub order: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub name: String,
    pub config: ItemConfig,
    pub impact_sounds: Vec<Uuid>,
    pub windup_sounds: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ItemWithSounds {
    #[serde(flatten)]
    pub item: ItemModel,
    pub impact_sounds_ids: Vec<Uuid>,
    pub windup_sounds_ids: Vec<Uuid>,
}

impl ItemModel {
    pub fn columns() -> [ItemsColumn; 5] {
        [
            ItemsColumn::Id,
            ItemsColumn::Name,
            ItemsColumn::Config,
            ItemsColumn::Order,
            ItemsColumn::CreatedAt,
        ]
    }

    pub async fn create(db: &DbPool, create: CreateItem) -> anyhow::Result<ItemModel> {
        let id = Uuid::new_v4();

        let model = ItemModel {
            id,
            name: create.name,
            config: create.config,
            order: 0,
            created_at: Utc::now(),
        };

        let config_value = serde_json::to_value(&model.config)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(ItemsTable)
                .columns([
                    ItemsColumn::Id,
                    ItemsColumn::Name,
                    ItemsColumn::Config,
                    ItemsColumn::Order,
                    ItemsColumn::CreatedAt,
                ])
                .values_panic([
                    model.id.into(),
                    model.name.clone().into(),
                    config_value.into(),
                    model.order.into(),
                    model.created_at.into(),
                ]),
        )
        .await?;

        model
            .append_sounds(db, &create.impact_sounds, SoundType::Impact)
            .await?;
        model
            .append_sounds(db, &create.windup_sounds, SoundType::Windup)
            .await?;

        Ok(model)
    }

    pub async fn all(db: &DbPool) -> DbResult<Vec<ItemModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns(ItemModel::columns())
                .from(ItemsTable)
                .order_by_columns([
                    (ItemsColumn::Order, Order::Asc),
                    (ItemsColumn::CreatedAt, Order::Desc),
                ]),
        )
        .await
    }

    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<ItemModel>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .columns(ItemModel::columns())
                .from(ItemsTable)
                .and_where(Expr::col(ItemsColumn::Id).eq(id)),
        )
        .await
    }

    pub async fn with_items_sounds(
        db: &DbPool,
        items: Vec<ItemModel>,
    ) -> DbResult<Vec<ItemWithSounds>> {
        // Collect IDs for loaded items
        let mut item_ids = Vec::new();

        // Map items to prep for sounds adding
        let mut items_with_sounds: Vec<ItemWithSounds> = items
            .into_iter()
            .map(|item| {
                item_ids.push(item.id);

                ItemWithSounds {
                    item,
                    impact_sounds_ids: Default::default(),
                    windup_sounds_ids: Default::default(),
                }
            })
            .collect();

        // Lookup the new sounds
        let rows: Vec<(Uuid, Uuid, SoundType)> = sql_query_all(
            db,
            Query::select()
                .columns([
                    ItemsSoundsColumn::ItemId,
                    ItemsSoundsColumn::SoundId,
                    ItemsSoundsColumn::SoundType,
                ])
                .from(ItemsSoundsTable)
                .and_where(Expr::col(ItemsSoundsColumn::ItemId).is_in(item_ids)),
        )
        .await?;

        // Connect the items and sounds
        for (item_id, sound_id, sound_type) in rows {
            // Find matching item
            let item = match items_with_sounds
                .iter_mut()
                .find(|item| item.item.id == item_id)
            {
                Some(item) => item,
                None => continue,
            };

            match sound_type {
                SoundType::Impact => item.impact_sounds_ids.push(sound_id),
                SoundType::Windup => item.windup_sounds_ids.push(sound_id),
            }
        }

        Ok(items_with_sounds)
    }

    pub async fn get_by_ids_with_sounds(
        db: &DbPool,
        ids: &[Uuid],
    ) -> DbResult<Vec<ItemWithSounds>> {
        // Request the items
        let items: Vec<ItemModel> = sql_query_all(
            db,
            Query::select()
                .columns(ItemModel::columns())
                .from(ItemsTable)
                .and_where(Expr::col(ItemsColumn::Id).is_in(ids.iter().copied())),
        )
        .await?;

        Self::with_items_sounds(db, items).await
    }

    pub async fn get_by_names_with_sounds(
        db: &DbPool,
        names: &[String],
        ignore_case: bool,
    ) -> DbResult<Vec<ItemWithSounds>> {
        // Request the items
        let items: Vec<ItemModel> = {
            let mut select = Query::select();

            select.columns(ItemModel::columns()).from(ItemsTable);

            if ignore_case {
                select.and_where(
                    // Convert stored name to lower case
                    Expr::expr(Func::lower(Expr::col(ItemsColumn::Name)))
                        // Compare with lowercase value
                        .is_in(names.iter().map(|value| value.to_lowercase())),
                );
            } else {
                select.and_where(Expr::col(ItemsColumn::Name).is_in(names));
            };

            sql_query_all(db, &select).await?
        };

        Self::with_items_sounds(db, items).await
    }

    /// Update the current item
    pub async fn update(&mut self, db: &DbPool, data: UpdateItem) -> anyhow::Result<()> {
        let mut update = Query::update();
        update
            .table(ItemsTable)
            .and_where(Expr::col(ItemsColumn::Id).eq(self.id));

        if let Some(name) = data.name {
            self.name = name.clone();
            update.value(ItemsColumn::Name, Expr::value(name));
        }

        if let Some(config) = data.config {
            let config_value = serde_json::to_value(&config)?;
            self.config = config;
            update.value(ItemsColumn::Config, Expr::value(config_value));
        }

        if let Some(order) = data.order {
            self.order = order;
            update.value(ItemsColumn::Order, Expr::value(order));
        }

        sql_exec(db, &update).await?;

        if let Some(impact_sounds) = data.impact_sounds {
            self.set_sounds(db, &impact_sounds, SoundType::Impact)
                .await?;
        }

        if let Some(windup_sounds) = data.windup_sounds {
            self.set_sounds(db, &windup_sounds, SoundType::Windup)
                .await?;
        }

        Ok(())
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let mut case = CaseStatement::new()
                // Use the current column value when not specified
                .finally(Expr::col(ItemsColumn::Order));

            // Add case for all updated values
            for order in order_chunk {
                case = case.case(
                    Expr::col(ItemsColumn::Id).eq(order.id),
                    Expr::value(order.order),
                );
            }

            sql_exec(
                db,
                Query::update()
                    .table(ItemsTable)
                    .value(ItemsColumn::Order, case),
            )
            .await?;
        }

        Ok(())
    }

    /// Sets the impact sounds for thsis item
    pub async fn set_sounds(
        &self,
        db: &DbPool,
        sound_ids: &[Uuid],
        sound_type: SoundType,
    ) -> DbResult<()> {
        // Delete any impact sounds not in the provided list
        sql_exec(
            db,
            Query::delete()
                .from_table(ItemsSoundsTable)
                .and_where(Expr::col(ItemsSoundsColumn::ItemId).eq(self.id))
                .and_where(
                    Expr::col(ItemsSoundsColumn::SoundId).is_not_in(sound_ids.iter().copied()),
                )
                .and_where(Expr::col(ItemsSoundsColumn::SoundType).eq(sound_type)),
        )
        .await?;

        self.append_sounds(db, sound_ids, sound_type).await?;

        Ok(())
    }

    /// Append impact sounds to the item
    pub async fn append_sounds(
        &self,
        db: &DbPool,
        sound_ids: &[Uuid],
        sound_type: SoundType,
    ) -> DbResult<()> {
        // Don't try and insert if theres no data
        if sound_ids.is_empty() {
            return Ok(());
        }

        sql_exec(
            db,
            Query::insert()
                .columns([
                    ItemsSoundsColumn::ItemId,
                    ItemsSoundsColumn::SoundId,
                    ItemsSoundsColumn::SoundType,
                ])
                .values_from_panic(
                    sound_ids
                        .iter()
                        .copied()
                        .map(|sound_id| [self.id.into(), sound_id.into(), sound_type.into()]),
                )
                .on_conflict(
                    OnConflict::columns([
                        ItemsSoundsColumn::ItemId,
                        ItemsSoundsColumn::SoundId,
                        ItemsSoundsColumn::SoundType,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .into_table(ItemsSoundsTable),
        )
        .await
    }

    pub async fn with_sounds(self, db: &DbPool) -> DbResult<ItemWithSounds> {
        // Map items to prep for sounds adding
        let mut item_with_sounds = ItemWithSounds {
            item: self,
            impact_sounds_ids: Default::default(),
            windup_sounds_ids: Default::default(),
        };

        // Lookup the sounds
        let rows: Vec<(Uuid, SoundType)> = sql_query_all(
            db,
            Query::select()
                .columns([ItemsSoundsColumn::SoundId, ItemsSoundsColumn::SoundType])
                .from(ItemsSoundsTable)
                .and_where(Expr::col(ItemsSoundsColumn::ItemId).eq(item_with_sounds.item.id)),
        )
        .await?;

        // Connect the items and sounds
        for (sound_id, sound_type) in rows {
            match sound_type {
                SoundType::Impact => item_with_sounds.impact_sounds_ids.push(sound_id),
                SoundType::Windup => item_with_sounds.windup_sounds_ids.push(sound_id),
            }
        }

        Ok(item_with_sounds)
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(ItemsTable)
                .and_where(Expr::col(ItemsColumn::Id).eq(self.id)),
        )
        .await
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "items")]
pub struct ItemsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ItemsColumn {
    Id,
    Name,
    Config,
    Order,
    CreatedAt,
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "items_sounds")]
pub struct ItemsSoundsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ItemsSoundsColumn {
    ItemId,
    SoundId,
    SoundType,
}
