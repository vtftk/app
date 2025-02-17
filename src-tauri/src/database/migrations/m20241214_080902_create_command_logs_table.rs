use super::{
    m20241208_060200_create_commands_table::{CommandsColumn, CommandsTable},
    Migration,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, SqliteQueryBuilder, Table};

pub struct CommandLogsMigration;

#[async_trait::async_trait]
impl Migration for CommandLogsMigration {
    fn name(&self) -> &str {
        "m20241214_080902_create_command_logs_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(CommandLogsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(CommandLogsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(CommandLogsColumn::CommandId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandLogsColumn::Level)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandLogsColumn::Message)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandLogsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                // Connect to commands table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_command_logs_command_id")
                        .from(CommandLogsTable, CommandLogsColumn::CommandId)
                        .to(CommandsTable, CommandsColumn::Id)
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
#[iden(rename = "command_logs")]
pub struct CommandLogsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum CommandLogsColumn {
    Id,
    CommandId,
    Level,
    Message,
    CreatedAt,
}
