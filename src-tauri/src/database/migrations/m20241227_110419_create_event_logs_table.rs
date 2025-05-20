use super::Migration;

pub struct EventLogsMigration;

#[async_trait::async_trait]
impl Migration for EventLogsMigration {
    fn name(&self) -> &str {
        "m20241227_110419_create_event_logs_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241227_110419_create_event_logs_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
