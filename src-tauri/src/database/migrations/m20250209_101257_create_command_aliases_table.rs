use super::Migration;

pub struct CommandAliasesMigration;

#[async_trait::async_trait]
impl Migration for CommandAliasesMigration {
    fn name(&self) -> &str {
        "m20250209_101257_create_command_aliases_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20250209_101257_create_command_aliases_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
