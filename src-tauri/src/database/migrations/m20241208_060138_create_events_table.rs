use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

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
                .col(
                    ColumnDef::new(EventsColumn::TriggerType)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventsColumn::Trigger)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventsColumn::Outcome)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventsColumn::Cooldown)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventsColumn::RequireRole)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EventsColumn::OutcomeDelay)
                        .integer()
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
    Trigger,
    Outcome,
    Cooldown,
    RequireRole,
    OutcomeDelay,
    Order,
    CreatedAt,
}
