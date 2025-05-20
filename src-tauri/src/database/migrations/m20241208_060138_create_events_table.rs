use super::Migration;

pub struct EventsMigration;

#[async_trait::async_trait]
impl Migration for EventsMigration {
    fn name(&self) -> &str {
        "m20241208_060138_create_events_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241208_060138_create_events_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
