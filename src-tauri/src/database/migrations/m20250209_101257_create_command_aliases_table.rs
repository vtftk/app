use super::{
    m20241208_060200_create_commands_table::{CommandsColumn, CommandsTable},
    Migration,
};
use sea_query::{
    ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, Index, SqliteQueryBuilder, Table,
};

pub struct CommandAliasesMigration;

#[async_trait::async_trait]
impl Migration for CommandAliasesMigration {
    fn name(&self) -> &str {
        "m20250209_101257_create_command_aliases_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(CommandAliasTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(CommandAliasColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(CommandAliasColumn::CommandId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandAliasColumn::Alias)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandAliasColumn::Order)
                        .integer()
                        .not_null(),
                )
                // Connect to commands table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_command_aliases_command_id")
                        .from(CommandAliasTable, CommandAliasColumn::CommandId)
                        .to(CommandsTable, CommandsColumn::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        // Index alias itself
        sqlx::query(
            &Index::create()
                .name("idx-command-alias")
                .table(CommandsTable)
                .col(CommandAliasColumn::Alias)
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "command_alias")]
pub struct CommandAliasTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum CommandAliasColumn {
    Id,
    CommandId,
    Alias,
    Order,
}
