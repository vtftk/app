use super::Migration;

pub struct KeyValueMigration;

#[async_trait::async_trait]
impl Migration for KeyValueMigration {
    fn name(&self) -> &str {
        "m20241212_114700_create_key_value_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241212_114700_create_key_value_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
