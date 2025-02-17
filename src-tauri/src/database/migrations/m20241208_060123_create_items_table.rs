//! # Create Items Table
//!
//! Migration that creates the "items" table which stores
//! throwable items

use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct ItemsMigration;

#[async_trait::async_trait]
impl Migration for ItemsMigration {
    fn name(&self) -> &str {
        "m20241208_060123_create_items_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(ItemsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(ItemsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ItemsColumn::Name).string().not_null())
                .col(ColumnDef::new(ItemsColumn::Config).json_binary().not_null())
                .col(
                    ColumnDef::new(ItemsColumn::Order)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(ItemsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "items")]
pub struct ItemsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ItemsColumn {
    Id,
    Name,
    Config,
    Order,
    CreatedAt,
}
