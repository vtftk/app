use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct ModelDataMigration;

#[async_trait::async_trait]
impl Migration for ModelDataMigration {
    fn name(&self) -> &str {
        "m20241208_060230_create_model_data_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(ModelDataTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(ModelDataColumn::Id)
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ModelDataColumn::Name).string().not_null())
                .col(
                    ColumnDef::new(ModelDataColumn::Calibration)
                        .json_binary()
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
#[iden(rename = "model_data")]
pub struct ModelDataTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ModelDataColumn {
    Id,
    Name,
    Calibration,
}
