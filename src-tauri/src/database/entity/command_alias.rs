use sea_query::{Expr, IdenStatic, Order, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_all},
    DbPool, DbResult,
};

use super::commands::CommandModel;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CommandAliasModel {
    /// Unique ID of the log
    pub id: Uuid,
    /// ID of the command
    pub command_id: Uuid,
    /// The alias
    pub alias: String,
    /// Order within the command aliases list
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandWithAliases {
    #[serde(flatten)]
    pub command: CommandModel,
    pub aliases: Vec<String>,
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

impl CommandAliasModel {
    /// Get all aliases for a specific command
    pub async fn get_aliases(db: &DbPool, command_id: Uuid) -> DbResult<Vec<String>> {
        let results: Vec<(String,)> = sql_query_all(
            db,
            Query::select()
                .columns([CommandAliasColumn::Alias])
                .from(CommandAliasTable)
                .and_where(Expr::col(CommandAliasColumn::CommandId).eq(command_id))
                .order_by(CommandAliasColumn::Order, Order::Asc),
        )
        .await?;

        Ok(results.into_iter().map(|(alias,)| alias).collect())
    }

    /// Sets all the aliases for a specific command
    /// (Removes the previous set and creates the new one)
    pub async fn set_aliases(db: &DbPool, command_id: Uuid, aliases: Vec<String>) -> DbResult<()> {
        // Delete all command aliases for the command
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandAliasTable)
                .and_where(Expr::col(CommandAliasColumn::CommandId).eq(command_id)),
        )
        .await?;

        // Insert new aliases
        sql_exec(
            db,
            Query::insert()
                .into_table(CommandAliasTable)
                .columns([
                    CommandAliasColumn::Id,
                    CommandAliasColumn::CommandId,
                    CommandAliasColumn::Alias,
                    CommandAliasColumn::Order,
                ])
                .values_from_panic(aliases.into_iter().enumerate().map(|(index, alias)| {
                    [
                        Uuid::new_v4().into(),
                        command_id.into(),
                        alias.into(),
                        (index as u32).into(),
                    ]
                })),
        )
        .await?;

        Ok(())
    }
}
