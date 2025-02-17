use super::{
    m20241208_060138_create_events_table::{EventsColumn, EventsTable},
    Migration,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, SqliteQueryBuilder, Table};

pub struct EventLogsMigration;

#[async_trait::async_trait]
impl Migration for EventLogsMigration {
    fn name(&self) -> &str {
        "m20241227_110419_create_event_logs_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(EventLogsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(EventLogsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(EventLogsColumn::EventId).uuid().not_null())
                .col(ColumnDef::new(EventLogsColumn::Level).integer().not_null())
                .col(ColumnDef::new(EventLogsColumn::Message).string().not_null())
                .col(
                    ColumnDef::new(EventLogsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                // Connect to events table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_event_logs_event_id")
                        .from(EventLogsTable, EventLogsColumn::EventId)
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
#[iden(rename = "event_logs")]
pub struct EventLogsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventLogsColumn {
    Id,
    EventId,
    Level,
    Message,
    CreatedAt,
}
