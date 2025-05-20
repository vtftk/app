use super::Migration;

pub struct ChatHistoryMigration;

#[async_trait::async_trait]
impl Migration for ChatHistoryMigration {
    fn name(&self) -> &str {
        "m20250124_082703_create_chat_history_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20250124_082703_create_chat_history_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
