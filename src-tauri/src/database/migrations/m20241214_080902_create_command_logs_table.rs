use super::Migration;

pub struct CommandLogsMigration;

#[async_trait::async_trait]
impl Migration for CommandLogsMigration {
    fn name(&self) -> &str {
        "m20241214_080902_create_command_logs_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241214_080902_create_command_logs_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
