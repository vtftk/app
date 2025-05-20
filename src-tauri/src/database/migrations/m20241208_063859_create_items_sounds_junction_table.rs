use super::Migration;

pub struct ItemsSoundsMigration;

#[async_trait::async_trait]
impl Migration for ItemsSoundsMigration {
    fn name(&self) -> &str {
        "m20241208_063859_create_items_sounds_junction_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!(
            "./sql/m20241208_063859_create_items_sounds_junction_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
