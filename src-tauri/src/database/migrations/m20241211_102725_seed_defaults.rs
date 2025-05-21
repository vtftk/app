use super::Migration;

pub struct SeedDefaultsMigration;

#[async_trait::async_trait]
impl Migration for SeedDefaultsMigration {
    fn name(&self) -> &str {
        "m20241211_102725_seed_defaults"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!("./sql/m20241211_102725_seed_defaults.sql"))
            .execute(db)
            .await?;
        Ok(())
    }
}
