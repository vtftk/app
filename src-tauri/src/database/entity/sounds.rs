use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database::{DbPool, DbResult};

use super::shared::UpdateOrdering;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, strum::Display, strum::EnumString, sqlx::Type,
)]
pub enum SoundType {
    Impact,
    Windup,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SoundModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Name of the sound
    pub name: String,
    /// Src URL for the image
    pub src: String,
    /// Volume of the sound 0-1
    pub volume: f32,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

/// Partial chunk of the sound model used for compute
/// purposes, excludes fields used by the UI
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartialSoundModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Src URL for the image
    pub src: String,
    /// Volume of the sound 0-1
    pub volume: f32,
}

#[derive(Debug, Deserialize)]
pub struct CreateSound {
    pub name: String,
    pub src: String,
    pub volume: f32,
}

#[derive(Default, Deserialize)]
pub struct UpdateSound {
    pub name: Option<String>,
    pub src: Option<String>,
    pub volume: Option<f32>,
}

impl SoundModel {
    /// Create a sound
    pub async fn create(db: &DbPool, create: CreateSound) -> anyhow::Result<SoundModel> {
        let id = Uuid::new_v4();
        let model = SoundModel {
            id,
            name: create.name,
            src: create.src,
            volume: create.volume,
            order: 0,
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO "sounds" ("id", "name", "src", "volume", "order", "created_at")
            VALUES (?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(model.id)
        .bind(model.name.as_str())
        .bind(model.src.as_str())
        .bind(model.volume)
        .bind(model.order)
        .bind(model.created_at)
        .execute(db)
        .await?;

        Ok(model)
    }

    /// Find a specific sound by ID
    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<SoundModel>> {
        sqlx::query_as(r#"SELECT * FROM "sounds" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    /// Find a specific sound by ID
    pub async fn get_by_id_partial(db: &DbPool, id: Uuid) -> DbResult<Option<PartialSoundModel>> {
        sqlx::query_as(r#"SELECT "id", "src", "volume" FROM "sounds" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    /// Find sounds with IDs present in the provided list
    pub async fn get_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<Vec<SoundModel>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat_n('?', ids.len()).join(",");
        let sql = format!(r#"SELECT * FROM "sounds" WHERE "id" IN ({placeholders})"#);
        let mut query = sqlx::query_as(&sql);

        for id in ids {
            query = query.bind(id);
        }

        let result = query.fetch_all(db).await?;
        Ok(result)
    }

    /// Find sounds with IDs present in the provided list
    pub async fn get_by_ids_partial(db: &DbPool, ids: &[Uuid]) -> DbResult<Vec<PartialSoundModel>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat_n('?', ids.len()).join(",");
        let sql =
            format!(r#"SELECT "id", "src", "volume" FROM "sounds" WHERE "id" IN ({placeholders})"#);
        let mut query = sqlx::query_as(&sql);

        for id in ids {
            query = query.bind(id);
        }

        let result = query.fetch_all(db).await?;
        Ok(result)
    }

    /// Find all sounds
    pub async fn all(db: &DbPool) -> DbResult<Vec<SoundModel>> {
        sqlx::query_as(r#"SELECT * FROM "sounds" ORDER BY "order" ASC, "created_at" DESC"#)
            .fetch_all(db)
            .await
    }

    /// Find all sounds with a matching name, optionally ignoring case
    pub async fn get_by_names(
        db: &DbPool,
        names: &[String],
        ignore_case: bool,
    ) -> DbResult<Vec<SoundModel>> {
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
        let placeholders = std::iter::repeat_n('?', names.len()).join(",");
        let sql = format!(r#"SELECT * FROM "sounds" WHERE {name_column} IN ({placeholders})"#);

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

    /// Update the current sound
    pub async fn update(&mut self, db: &DbPool, data: UpdateSound) -> DbResult<()> {
        let name = data.name.unwrap_or_else(|| self.name.clone());
        let src = data.src.unwrap_or_else(|| self.src.clone());
        let volume = data.volume.unwrap_or(self.volume);

        sqlx::query(r#"UPDATE "sounds" SET "name" = ?, "src" = ?, "volume" = ? WHERE "id" = ?"#)
            .bind(name.as_str())
            .bind(src.as_str())
            .bind(volume)
            .bind(self.id)
            .execute(db)
            .await?;

        self.name = name;
        self.src = src;
        self.volume = volume;

        Ok(())
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let cases = std::iter::repeat_n("WHEN ? THEN ?", order_chunk.len())
                .join(" ");

            let sql = format!(
                r#"
                UPDATE "sounds"
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

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "sounds" WHERE "id" = ?"#)
            .bind(self.id)
            .execute(db)
            .await?;

        Ok(())
    }
}
