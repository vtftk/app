use super::{shared::UpdateOrdering, sounds::SoundType};
use crate::database::{DbErr, DbPool, DbResult};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

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
    pub async fn create(db: &DbPool, create: CreateItem) -> DbResult<ItemModel> {
        let id = Uuid::new_v4();

        let model = ItemModel {
            id,
            name: create.name,
            config: create.config,
            order: 0,
            created_at: Utc::now(),
        };

        let config_value =
            serde_json::to_value(&model.config).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(
            r#"
            INSERT INTO "items" ("id", "name", "config", "order", "created_at")
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.name.as_str())
        .bind(config_value)
        .bind(model.order)
        .bind(model.created_at)
        .execute(db)
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
        sqlx::query_as(r#"SELECT * FROM "items" ORDER BY "order" ASC, "created_at" DESC"#)
            .fetch_all(db)
            .await
    }

    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<ItemModel>> {
        sqlx::query_as(r#"SELECT * FROM "items" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    async fn get_items_sounds(
        db: &DbPool,
        item_ids: &[Uuid],
    ) -> DbResult<Vec<(Uuid, Uuid, SoundType)>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat('?').take(item_ids.len()).join(",");
        let sql = format!(
            r#"SELECT "item_id", "sound_id", "sound_type" FROM "items_sounds" WHERE "item_id" IN ({placeholders})"#
        );
        let mut query = sqlx::query_as(&sql);

        for id in item_ids {
            query = query.bind(id);
        }

        let result = query.fetch_all(db).await?;

        Ok(result)
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
        let rows: Vec<(Uuid, Uuid, SoundType)> = Self::get_items_sounds(db, &item_ids).await?;

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
        let items = Self::get_by_ids(db, ids).await?;
        Self::with_items_sounds(db, items).await
    }

    async fn get_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<Vec<ItemModel>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat('?').take(ids.len()).join(",");
        let sql = format!(r#"SELECT * FROM "items" WHERE "id" IN ({placeholders})"#);
        let mut query = sqlx::query_as(&sql);

        for id in ids {
            query = query.bind(id);
        }

        let result = query.fetch_all(db).await?;

        Ok(result)
    }

    async fn get_by_names(
        db: &DbPool,
        names: &[String],
        ignore_case: bool,
    ) -> DbResult<Vec<ItemModel>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }

        let name_column = if ignore_case {
            // When ignoring case wrap the name in the LOWER function
            r#"LOWER("name")"#
        } else {
            r#""name""#
        };

        // Create the value placeholders for the names
        let placeholders = std::iter::repeat('?').take(names.len()).join(",");
        let sql = format!(r#"SELECT * FROM "items" WHERE {name_column} IN ({placeholders})"#);

        let mut query = sqlx::query_as(&sql);

        if ignore_case {
            for name in names {
                query = query.bind(name.to_lowercase());
            }
        } else {
            for name in names {
                query = query.bind(name);
            }
        }

        let result = query.fetch_all(db).await?;
        Ok(result)
    }

    pub async fn get_by_names_with_sounds(
        db: &DbPool,
        names: &[String],
        ignore_case: bool,
    ) -> DbResult<Vec<ItemWithSounds>> {
        // Request the items
        let items = Self::get_by_names(db, names, ignore_case).await?;
        Self::with_items_sounds(db, items).await
    }

    /// Update the current item
    pub async fn update(&mut self, db: &DbPool, data: UpdateItem) -> anyhow::Result<()> {
        let name = data.name.unwrap_or_else(|| self.name.clone());
        let config = data.config.unwrap_or_else(|| self.config.clone());
        let config_value =
            serde_json::to_value(&config).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(r#"UPDATE "items" SET "name" = ?, "config" = ? WHERE "id" = ?"#)
            .bind(name.as_str())
            .bind(config_value)
            .bind(self.id)
            .execute(db)
            .await?;

        self.name = name;
        self.config = config;

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
            let cases = std::iter::repeat("WHEN ? THEN ?")
                .take(order_chunk.len())
                .join(" ");

            let sql = format!(
                r#"
                UPDATE "items"
                SET "order" = CASE "id"
                    {cases}
                    ELSE "order"
                END
            "#
            );

            let mut query = sqlx::query(&sql);

            for order in order_chunk {
                query = query.bind(order.id).bind(order.order);
            }

            query.execute(db).await?;
        }

        Ok(())
    }

    /// Sets the impact sounds for this item
    pub async fn set_sounds(
        &self,
        db: &DbPool,
        sound_ids: &[Uuid],
        sound_type: SoundType,
    ) -> DbResult<()> {
        // Delete any sounds already attached
        self.delete_sounds_by_type(db, sound_type).await?;
        self.append_sounds(db, sound_ids, sound_type).await?;

        Ok(())
    }

    /// Delete sounds of a specific type for a specific item
    async fn delete_sounds_by_type(&self, db: &DbPool, sound_type: SoundType) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "items_sounds" WHERE "item_id" = ? OR "sound_type" = ?"#)
            .bind(self.id)
            .bind(sound_type)
            .execute(db)
            .await?;
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

        // Generate the placeholders required to insert values
        let values_sets = std::iter::repeat("(?,?,?)").take(sound_ids.len()).join(",");
        let sql = format!(
            r#"
            INSERT INTO "items_sounds" ("item_id", "sound_id", "sound_type") 
            VALUES {values_sets}
            ON CONFLICT("item_id", "sound_id", "sound_type") DO NOTHING
            "#
        );

        let mut query = sqlx::query(&sql);

        for sound_id in sound_ids {
            query = query.bind(self.id).bind(sound_id).bind(sound_type);
        }

        query.execute(db).await?;
        Ok(())
    }

    pub async fn with_sounds(self, db: &DbPool) -> DbResult<ItemWithSounds> {
        // Map items to prep for sounds adding
        let mut item_with_sounds = ItemWithSounds {
            item: self,
            impact_sounds_ids: Default::default(),
            windup_sounds_ids: Default::default(),
        };

        // Lookup the sounds
        let rows: Vec<(Uuid, SoundType)> = sqlx::query_as(
            r#"SELECT "sound_id", "sound_type" FROM "items_sounds" WHERE "item_id" = ?"#,
        )
        .bind(item_with_sounds.item.id)
        .fetch_all(db)
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
        sqlx::query(r#"DELETE FROM "items" WHERE "id" = ?"#)
            .bind(self.id)
            .execute(db)
            .await?;

        Ok(())
    }
}
