use crate::database::{DbPool, DbResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretsModel {
    pub key: String,
    pub value: String,
    #[sqlx(jsn)]
    pub metadata: serde_json::Value,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SetSecret {
    pub key: String,
    pub value: String,
    pub metadata: serde_json::Value,
}

impl SecretsModel {
    /// Create a new sound
    pub async fn set(db: &DbPool, create: SetSecret) -> DbResult<()> {
        let created_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO "secrets" ("key", "value", "metadata", "created_at")
            VALUES (?, ?, ?, ?)
            ON CONFLICT(Id) DO UPDATE SET
                "value" = excluded."value",
                "metadata" = excluded."metadata",
                "created_at" = excluded."created_at"
        "#,
        )
        .bind(create.key)
        .bind(create.value)
        .bind(create.metadata)
        .bind(created_at)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Find a specific key value by key
    pub async fn get_by_key(db: &DbPool, key: &str) -> DbResult<Option<Self>> {
        sqlx::query_as(r#"SELECT * FROM "secrets" WHERE "key" = ?"#)
            .bind(key)
            .fetch_optional(db)
            .await
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "secrets" WHERE "key" = ?"#)
            .bind(key)
            .execute(db)
            .await?;

        Ok(())
    }
}
