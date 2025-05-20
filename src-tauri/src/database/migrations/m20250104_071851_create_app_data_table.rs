use super::Migration;

pub struct AppDataMigration;

#[async_trait::async_trait]
impl Migration for AppDataMigration {
    fn name(&self) -> &str {
        "m20250104_071851_create_app_data_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20250104_071851_create_app_data_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
