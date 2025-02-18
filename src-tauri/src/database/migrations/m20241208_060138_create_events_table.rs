use super::Migration;
use sea_query::{ColumnDef, Expr, IdenStatic, Index, SqliteQueryBuilder, Table};

pub struct EventsMigration;

#[async_trait::async_trait]
impl Migration for EventsMigration {
    fn name(&self) -> &str {
        "m20241208_060138_create_events_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(EventsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(EventsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(EventsColumn::Enabled).boolean().not_null())
                .col(ColumnDef::new(EventsColumn::Name).string().not_null())
                // "trigger_type" is stored virtual column derived from the "type" discriminated union variant
                // identifier for "trigger" used for searching based on type without needing to parse all the JSON
                .col(
                    ColumnDef::new(EventsColumn::TriggerType)
                        .string()
                        .not_null()
                        .generated(
                            Expr::cust("json_extract(\"config\", '$.trigger.type')"),
                            true,
                        ),
                )
                .col(
                    ColumnDef::new(EventsColumn::Config)
                        .json_binary()
                        .not_null(),
                )
                .col(ColumnDef::new(EventsColumn::Order).integer().not_null())
                .col(
                    ColumnDef::new(EventsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        // When triggering events its very common to lookup events for a
        // specific type that are enabled, so we index that
        sqlx::query(
            &Index::create()
                .name("idx-events-trigger-type-enabled")
                .table(EventsTable)
                .col(EventsColumn::TriggerType)
                .col(EventsColumn::Enabled)
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "events")]
pub struct EventsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventsColumn {
    Id,
    Enabled,
    Name,
    TriggerType,
    Config,
    Order,
    CreatedAt,
}
