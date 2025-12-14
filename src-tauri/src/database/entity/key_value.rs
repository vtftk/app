use crate::database::{DbPool, DbResult};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct KeyValueModel {
    /// Key for the key value pair
    pub key: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub ty: KeyValueType,
    pub value: String,
}

/// Key value type
#[derive(Debug, Copy, Clone, Serialize, Deserialize, EnumString, Display, sqlx::Type)]
pub enum KeyValueType {
    /// Plain text is stored
    Text,
    /// Number is stored as plain text
    Number,
    /// Object is stored as JSON
    Object,
    /// Array is stored as JSON
    Array,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyValue {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub ty: KeyValueType,
}

impl KeyValueModel {
    /// Create a new sound
    pub async fn create(db: &DbPool, create: CreateKeyValue) -> DbResult<()> {
        sqlx::query(
            r#"
            INSERT INTO "key_value" ("key", "value", "type")
            VALUES (?, ?, ?)
            ON CONFLICT("key") DO UPDATE SET
                "value" = excluded."value",
                "type" = excluded."type"
        "#,
        )
        .bind(create.key)
        .bind(create.value)
        .bind(create.ty)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Find a specific key value by key
    pub async fn get_by_key(db: &DbPool, key: &str) -> DbResult<Option<Self>> {
        sqlx::query_as(r#"SELECT * FROM "key_value" WHERE "key" = ?"#)
            .bind(key)
            .fetch_optional(db)
            .await
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "key_value" WHERE "key" = ?"#)
            .bind(key)
            .execute(db)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_create() {}

    #[tokio::test]
    async fn test_create_existing() {}

    #[tokio::test]
    async fn test_get_by_key_unknown() {}

    #[tokio::test]
    async fn test_get_by_key_known() {}

    #[tokio::test]
    async fn test_delete_by_key() {}
}
