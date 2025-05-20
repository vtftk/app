use super::Migration;

pub struct SecretsMigration;

#[async_trait::async_trait]
impl Migration for SecretsMigration {
    fn name(&self) -> &str {
        "m20250216_140137_create_secrets_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20250216_140137_create_secrets_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
