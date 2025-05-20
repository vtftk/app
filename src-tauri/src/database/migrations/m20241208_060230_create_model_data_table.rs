use super::Migration;

pub struct ModelDataMigration;

#[async_trait::async_trait]
impl Migration for ModelDataMigration {
    fn name(&self) -> &str {
        "m20241208_060230_create_model_data_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241208_060230_create_model_data_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
