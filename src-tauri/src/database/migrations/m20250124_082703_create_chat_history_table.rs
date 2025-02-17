use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct ChatHistoryMigration;

#[async_trait::async_trait]
impl Migration for ChatHistoryMigration {
    fn name(&self) -> &str {
        "m20250124_082703_create_chat_history_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(ChatHistoryTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(ChatHistoryColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(ChatHistoryColumn::UserId)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ChatHistoryColumn::Message)
                        .string()
                        .not_null(),
                )
                .col(ColumnDef::new(ChatHistoryColumn::Cheer).integer().null())
                .col(
                    ColumnDef::new(ChatHistoryColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
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
