use super::Migration;

pub struct SoundsMigration;

#[async_trait::async_trait]
impl Migration for SoundsMigration {
    fn name(&self) -> &str {
        "m20241208_060144_create_sounds_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241208_060144_create_sounds_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
