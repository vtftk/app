use super::Migration;

pub struct CommandExecutionsMigration;

#[async_trait::async_trait]
impl Migration for CommandExecutionsMigration {
    fn name(&self) -> &str {
        "m20241210_082316_create_command_executions_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241210_082316_create_command_executions_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
