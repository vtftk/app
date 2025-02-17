use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct SoundsMigration;

#[async_trait::async_trait]
impl Migration for SoundsMigration {
    fn name(&self) -> &str {
        "m20241208_060144_create_sounds_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(SoundsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(SoundsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(SoundsColumn::Name).string().not_null())
                .col(ColumnDef::new(SoundsColumn::Src).string().not_null())
                .col(ColumnDef::new(SoundsColumn::Volume).float().not_null())
                .col(ColumnDef::new(SoundsColumn::Order).integer().not_null())
                .col(
                    ColumnDef::new(SoundsColumn::CreatedAt)
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
#[iden(rename = "sounds")]
pub struct SoundsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum SoundsColumn {
    Id,
    Name,
    Src,
    Volume,
    Order,
    CreatedAt,
}
