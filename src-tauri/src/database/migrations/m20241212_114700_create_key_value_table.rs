use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct KeyValueMigration;

#[async_trait::async_trait]
impl Migration for KeyValueMigration {
    fn name(&self) -> &str {
        "m20241212_114700_create_key_value_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(KeyValueTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(KeyValueColumn::Key)
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(KeyValueColumn::Value).string().not_null())
                .col(ColumnDef::new(KeyValueColumn::Type).string().not_null())
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "key_value")]
pub struct KeyValueTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum KeyValueColumn {
    Key,
    Value,
    Type,
}
