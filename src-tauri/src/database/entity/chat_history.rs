use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use twitch_api::types::UserId;
use uuid::Uuid;

use crate::database::{DbPool, DbResult};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct ChatHistoryModel {
    /// Unique ID of the log
    pub id: Uuid,
    /// ID of the twitch user
    pub user_id: String,
    /// Chat message data
    pub message: String,
    /// Optional cheer amount
    pub cheer: Option<u32>,
    /// Creation time of the chat message
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateChatHistory {
    /// Unique ID of the log
    pub id: Uuid,
    /// ID of the twitch user
    pub user_id: String,
    /// Chat message data
    pub message: String,
    /// Optional cheer amount
    pub cheer: Option<u32>,
    /// Creation time of the chat message
    pub created_at: DateTime<Utc>,
}

impl ChatHistoryModel {
    /// Create a new chat history item
    pub async fn create(db: &DbPool, create: CreateChatHistory) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "chat_history" ("id", "user_id", "message", "cheer", "created_at") VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(create.id)
        .bind(create.user_id)
        .bind(create.message)
        .bind(create.cheer)
        .bind(create.created_at)
        .execute(db).await?;
        Ok(())
    }

    /// Estimates the size in bytes that the current chat history is taking up
    pub async fn estimate_size(db: &DbPool) -> DbResult<u32> {
        let result: (u32,) =
            sqlx::query_as(r#"SELECT COALESCE(SUM(LENGTH("message")), 0) FROM "chat_history" "#)
                .fetch_one(db)
                .await?;
        Ok(result.0)
    }

    /// Get the number of chat history messages since `start_date` excluding those
    /// where the user id matches `exclude_id` (When provided)
    pub async fn count_since(
        db: &DbPool,
        start_date: DateTime<Utc>,
        exclude_id: Option<UserId>,
    ) -> DbResult<u32> {
        let result: (u32,) = if let Some(exclude_id) = exclude_id {
            sqlx::query_as(
                r#"SELECT * FROM "chat_history" WHERE "created_at" > ? AND "user_id" != ?"#,
            )
            .bind(start_date)
            .bind(exclude_id.as_str())
            .fetch_one(db)
            .await?
        } else {
            sqlx::query_as(r#"SELECT COUNT(*) FROM "chat_history" WHERE "created_at" > ?"#)
                .bind(start_date)
                .fetch_one(db)
                .await?
        };

        Ok(result.0)
    }

    /// Deletes all chat history that happened before the provided `start_time`.
    /// Used to clean out old chat history
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "chat_history" WHERE "created_at" < ?"#)
            .bind(start_date)
            .execute(db)
            .await?;

        Ok(())
    }
}
