use chrono::{DateTime, Utc};
use sea_query::{Asterisk, Expr, Func, IdenStatic, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use twitch_api::types::UserId;
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_one_single},
    DbPool, DbResult,
};

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
        sql_exec(
            db,
            Query::insert()
                .into_table(ChatHistoryTable)
                .columns([
                    ChatHistoryColumn::Id,
                    ChatHistoryColumn::UserId,
                    ChatHistoryColumn::Message,
                    ChatHistoryColumn::Cheer,
                    ChatHistoryColumn::CreatedAt,
                ])
                .values_panic([
                    create.id.into(),
                    create.user_id.into(),
                    create.message.into(),
                    create.cheer.into(),
                    create.created_at.into(),
                ]),
        )
        .await
    }

    /// Estimates the size in bytes that the current chat history is taking up
    pub async fn estimate_size(db: &DbPool) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select().from(ChatHistoryTable).expr(Func::coalesce([
                // Get total length of all metadata text
                Func::sum(Func::char_length(Expr::col(ChatHistoryColumn::Message))).into(),
                // Fallback to zero
                Expr::value(0),
            ])),
        )
        .await
    }

    /// Get the number of chat history messages since `start_date` excluding those
    /// where the user id matches `exclude_id` (When provided)
    pub async fn count_since(
        db: &DbPool,
        start_date: DateTime<Utc>,
        exclude_id: Option<UserId>,
    ) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select()
                .from(ChatHistoryTable)
                .expr(Func::count(Expr::col(Asterisk)))
                .and_where(Expr::col(ChatHistoryColumn::CreatedAt).gt(Expr::value(start_date)))
                .and_where_option(exclude_id.map(|exclude_id| {
                    Expr::col(ChatHistoryColumn::UserId).ne(exclude_id.as_str())
                })),
        )
        .await
    }

    /// Deletes all chat history that happened before the provided `start_time`.
    /// Used to clean out old chat history
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(ChatHistoryTable)
                .and_where(Expr::col(ChatHistoryColumn::CreatedAt).lt(start_date)),
        )
        .await
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "chat_history")]
pub struct ChatHistoryTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ChatHistoryColumn {
    /// Twitch message ID
    Id,
    /// Twitch user ID
    UserId,
    /// Twitch message
    Message,
    /// Associated cheer amount (Optional)
    Cheer,
    /// Creation time of the chat message
    CreatedAt,
}
