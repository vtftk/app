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
                r#"SELECT COUNT(*) FROM "chat_history" WHERE "created_at" > ? AND "user_id" != ?"#,
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

#[cfg(test)]
mod test {
    use chrono::{Days, Utc};
    use uuid::Uuid;

    use crate::database::{
        entity::chat_history::{ChatHistoryModel, CreateChatHistory},
        mock_database,
    };

    #[tokio::test]
    async fn test_create() {
        let db = mock_database().await;

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_estimate_size() {
        let db = mock_database().await;
        let message_1 = "a".repeat(50).to_string();
        let message_2 = "b".repeat(50).to_string();
        let message_size = message_1.len() + message_2.len();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test".to_string(),
                message: message_1.to_string(),
                cheer: None,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: message_2.to_string(),
                cheer: None,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        let est_size = ChatHistoryModel::estimate_size(&db).await.unwrap();
        assert_eq!(est_size as usize, message_size);
    }

    #[tokio::test]
    async fn test_estimate_size_empty() {
        let db = mock_database().await;
        let est_size = ChatHistoryModel::estimate_size(&db).await.unwrap();
        assert_eq!(est_size, 0);
    }

    #[tokio::test]
    async fn test_count_since() {
        let db = mock_database().await;

        let first_time = Utc::now();
        let second_time = Utc::now().checked_add_days(Days::new(50)).unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        let count = ChatHistoryModel::count_since(
            &db,
            second_time.checked_sub_days(Days::new(1)).unwrap(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_count_since_excluding() {
        let db = mock_database().await;

        let first_time = Utc::now();
        let second_time = Utc::now().checked_add_days(Days::new(50)).unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_3".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_3".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        let count = ChatHistoryModel::count_since(
            &db,
            second_time.checked_sub_days(Days::new(1)).unwrap(),
            Some("test_2".into()),
        )
        .await
        .unwrap();

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_count_since_empty() {
        let db = mock_database().await;

        let first_time = Utc::now();
        let second_time = Utc::now().checked_add_days(Days::new(50)).unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        let count = ChatHistoryModel::count_since(
            &db,
            second_time.checked_add_days(Days::new(1)).unwrap(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_since_empty_excluding() {
        let db = mock_database().await;

        let first_time = Utc::now();
        let second_time = Utc::now().checked_add_days(Days::new(50)).unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        let count = ChatHistoryModel::count_since(
            &db,
            second_time.checked_sub_days(Days::new(1)).unwrap(),
            Some("test_2".into()),
        )
        .await
        .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_before() {
        let db = mock_database().await;

        let first_time = Utc::now();
        let second_time = Utc::now().checked_add_days(Days::new(50)).unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: first_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::create(
            &db,
            CreateChatHistory {
                id: Uuid::new_v4(),
                user_id: "test_2".to_string(),
                message: "test".to_string(),
                cheer: None,
                created_at: second_time,
            },
        )
        .await
        .unwrap();

        ChatHistoryModel::delete_before(&db, second_time.checked_sub_days(Days::new(1)).unwrap())
            .await
            .unwrap();

        let count = ChatHistoryModel::count_since(
            &db,
            first_time.checked_sub_days(Days::new(1)).unwrap(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(count, 3);
    }
}
