use super::Migration;

pub struct EventExecutionsMigration;

#[async_trait::async_trait]
impl Migration for EventExecutionsMigration {
    fn name(&self) -> &str {
        "m20241210_082256_create_event_executions_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241210_082256_create_event_executions_table.sql"
        ))
        .execute(db)
        .await?;
        Ok(())
    }
}
