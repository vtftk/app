use super::{
    m20241208_060200_create_commands_table::{CommandsColumn, CommandsTable},
    Migration,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, SqliteQueryBuilder, Table};

pub struct CommandExecutionsMigration;

#[async_trait::async_trait]
impl Migration for CommandExecutionsMigration {
    fn name(&self) -> &str {
        "m20241210_082316_create_command_executions_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(CommandExecutionsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(CommandExecutionsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(CommandExecutionsColumn::CommandId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandExecutionsColumn::Metadata)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandExecutionsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                // Connect to commands table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_command_executions_event_id")
                        .from(CommandExecutionsTable, CommandExecutionsColumn::CommandId)
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
#[iden(rename = "command_executions")]
pub struct CommandExecutionsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum CommandExecutionsColumn {
    Id,
    CommandId,
    Metadata,
    CreatedAt,
}
