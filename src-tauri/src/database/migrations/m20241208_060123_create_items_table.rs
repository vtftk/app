//! # Create Items Table
//!
//! Migration that creates the "items" table which stores
//! throwable items

use super::Migration;

pub struct ItemsMigration;

#[async_trait::async_trait]
impl Migration for ItemsMigration {
    fn name(&self) -> &str {
        "m20241208_060123_create_items_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(include_str!(
            "./sql/m20241208_060123_create_items_table.sql"
        ))
        .execute(db)
        .await?;

        Ok(())
    }
}
