use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct CommandsMigration;

#[async_trait::async_trait]
impl Migration for CommandsMigration {
    fn name(&self) -> &str {
        "m20241208_060200_create_commands_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(CommandsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(CommandsColumn::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(CommandsColumn::Enabled).boolean().not_null())
                .col(ColumnDef::new(CommandsColumn::Name).string().not_null())
                .col(ColumnDef::new(CommandsColumn::Command).text().not_null())
                .col(
                    ColumnDef::new(CommandsColumn::Outcome)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandsColumn::Cooldown)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CommandsColumn::RequireRole)
                        .string()
                        .not_null(),
                )
                .col(ColumnDef::new(CommandsColumn::Order).integer().not_null())
                .col(
                    ColumnDef::new(CommandsColumn::CreatedAt)
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
#[iden(rename = "commands")]
pub struct CommandsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum CommandsColumn {
    Id,
    Enabled,
    Name,
    Command,
    Outcome,
    Cooldown,
    RequireRole,
    Order,
    CreatedAt,
}
