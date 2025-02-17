use chrono::{DateTime, Utc};
use sea_query::{Alias, Asterisk, Expr, Func, IdenStatic, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use twitch_api::types::UserId;
use uuid::Uuid;

use crate::database::{DbPool, DbResult};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
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
    /// Create a new script
    pub async fn create(db: &DbPool, create: CreateChatHistory) -> DbResult<()> {
        let (sql, values) = Query::insert()
            .into_table(ChatHistoryTable)
            .columns(ChatHistoryModel::columns())
            .values_panic([
                create.id.into(),
                create.user_id.into(),
                create.message.into(),
                create.cheer.into(),
                create.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(())
    }

    /// Estimates the size in bytes that the current chat history is taking up
    pub async fn estimate_size(db: &DbPool) -> DbResult<u32> {
        #[derive(Default, FromRow)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let (sql, values) = Query::select()
            .from(ChatHistoryTable)
            .expr_as(
                Func::sum(Func::char_length(Expr::col(ChatHistoryColumn::Message))),
                Alias::new("total_message_length"),
            )
            .build_sqlx(SqliteQueryBuilder);

        let result: PartialModel = sqlx::query_as_with(&sql, values).fetch_one(db).await?;
        Ok(result.total_message_length.unwrap_or_default())
    }

    pub async fn count_since(
        db: &DbPool,
        start_date: DateTime<Utc>,
        exclude_id: Option<UserId>,
    ) -> DbResult<u32> {
        let mut select = Query::select();

        #[derive(FromRow)]
        struct Count {
            count: u32,
        }

        select
            .from(ChatHistoryTable)
            .columns(ChatHistoryModel::columns())
            .and_where(Expr::col(ChatHistoryColumn::CreatedAt).gt(Expr::value(start_date)))
            .expr_as(Func::count(Expr::col(Asterisk)), Alias::new("count"));

        if let Some(exclude_id) = exclude_id {
            select.and_where(Expr::col(ChatHistoryColumn::UserId).ne(exclude_id.as_str()));
        }

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        let result: Count = sqlx::query_as_with(&sql, values).fetch_one(db).await?;

        Ok(result.count)
    }

    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(ChatHistoryTable)
            .and_where(Expr::col(ChatHistoryColumn::CreatedAt).lt(start_date))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub fn columns() -> [ChatHistoryColumn; 5] {
        [
            ChatHistoryColumn::Id,
            ChatHistoryColumn::UserId,
            ChatHistoryColumn::Message,
            ChatHistoryColumn::Cheer,
            ChatHistoryColumn::CreatedAt,
        ]
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
