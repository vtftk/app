use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct AppDataMigration;

#[async_trait::async_trait]
impl Migration for AppDataMigration {
    fn name(&self) -> &str {
        "m20250104_071851_create_app_data_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(AppDataTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(AppDataColumn::Id)
                        .integer()
                        .auto_increment()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(AppDataColumn::Data).json_binary().not_null())
                .col(
                    ColumnDef::new(AppDataColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AppDataColumn::LastModifiedAt)
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
#[iden(rename = "app_data")]
pub struct AppDataTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum AppDataColumn {
    Id,
    Data,
    CreatedAt,
    LastModifiedAt,
}
