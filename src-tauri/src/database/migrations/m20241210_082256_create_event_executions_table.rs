use super::{
    m20241208_060138_create_events_table::{EventsColumn, EventsTable},
    Migration,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, SqliteQueryBuilder, Table};

pub struct EventExecutionsMigration;

#[async_trait::async_trait]
impl Migration for EventExecutionsMigration {
    fn name(&self) -> &str {
        "m20241210_082256_create_event_executions_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(EventExecutionsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(EventExecutionsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(EventExecutionsColumn::EventId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventExecutionsColumn::Metadata)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventExecutionsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                // Connect to events table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_event_executions_event_id")
                        .from(EventExecutionsTable, EventExecutionsColumn::EventId)
                        .to(EventsTable, EventsColumn::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "event_executions")]
pub struct EventExecutionsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventExecutionsColumn {
    Id,
    EventId,
    Metadata,
    CreatedAt,
}
