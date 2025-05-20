use super::Migration;

pub struct CommandsMigration;

#[async_trait::async_trait]
impl Migration for CommandsMigration {
    fn name(&self) -> &str {
        "m20241208_060200_create_commands_table"
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
