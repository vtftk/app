use super::{
    command_alias::{CommandAliasColumn, CommandAliasModel, CommandAliasTable},
    shared::{MinimumRequireRole, UpdateOrdering},
};
use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_maybe_one},
    DbPool, DbResult,
};
use chrono::{DateTime, Utc};
use sea_query::{CaseStatement, Condition, Expr, IdenStatic, Order, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommandModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Whether the command is enabled and runnable
    pub enabled: bool,
    /// Name of the command
    pub name: String,
    /// The command to trigger when entered
    pub command: String,
    /// Configuration for the command
    #[sqlx(json)]
    pub config: CommandConfig,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// The outcome of the command
    pub outcome: CommandOutcome,
    /// Cooldown between each trigger of the command
    pub cooldown: CommandCooldown,
    /// Minimum required role to trigger the command
    pub require_role: MinimumRequireRole,
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
    Config,
    Order,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandOutcome {
    Template { message: String },
    Script { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandCooldown {
    pub enabled: bool,
    pub duration: u32,
    pub per_user: bool,
}

impl Default for CommandCooldown {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 1000,
            per_user: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCommand {
    pub enabled: bool,
    pub name: String,
    pub command: String,
    pub config: CommandConfig,
    pub aliases: Vec<String>,
}

#[derive(Default, Deserialize)]
pub struct UpdateCommand {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub command: Option<String>,
    pub config: Option<CommandConfig>,
    pub order: Option<u32>,
    pub aliases: Option<Vec<String>>,
}

impl CommandModel {
    pub fn columns() -> [CommandsColumn; 7] {
        [
            CommandsColumn::Id,
            CommandsColumn::Enabled,
            CommandsColumn::Name,
            CommandsColumn::Command,
            CommandsColumn::Config,
            CommandsColumn::Order,
            CommandsColumn::CreatedAt,
        ]
    }

    /// Create a new sound
    pub async fn create(db: &DbPool, create: CreateCommand) -> anyhow::Result<CommandModel> {
        let id = Uuid::new_v4();
        let model = CommandModel {
            id,
            enabled: create.enabled,
            name: create.name,
            command: create.command.to_lowercase(),
            config: create.config,
            order: 0,
            created_at: Utc::now(),
        };

        let config_value = serde_json::to_value(&model.config)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(CommandsTable)
                .columns([
                    CommandsColumn::Id,
                    CommandsColumn::Enabled,
                    CommandsColumn::Name,
                    CommandsColumn::Command,
                    CommandsColumn::Config,
                    CommandsColumn::Order,
                    CommandsColumn::CreatedAt,
                ])
                .values_panic([
                    model.id.into(),
                    model.enabled.into(),
                    model.name.as_str().into(),
                    model.command.as_str().into(),
                    config_value.into(),
                    model.order.into(),
                    model.created_at.into(),
                ]),
        )
        .await?;

        // Set the command aliases
        CommandAliasModel::set_aliases(db, id, create.aliases).await?;

        Ok(model)
    }

    /// Find commands by the actual command trigger word
    /// and only commands that are enabled
    pub async fn get_by_command(db: &DbPool, command: &str) -> DbResult<Vec<CommandModel>> {
        sql_query_all(
            db,
            Query::select()
                .from(CommandsTable)
                .columns([
                    (CommandsTable, CommandsColumn::Id),
                    (CommandsTable, CommandsColumn::Enabled),
                    (CommandsTable, CommandsColumn::Name),
                    (CommandsTable, CommandsColumn::Command),
                    (CommandsTable, CommandsColumn::Config),
                    (CommandsTable, CommandsColumn::Order),
                    (CommandsTable, CommandsColumn::CreatedAt),
                ])
                .left_join(
                    CommandAliasTable,
                    Expr::col((CommandsTable, CommandsColumn::Id))
                        .equals((CommandAliasTable, CommandAliasColumn::CommandId)),
                )
                .and_where(Expr::col((CommandsTable, CommandsColumn::Enabled)).eq(true))
                .cond_where(
                    Condition::any()
                        .add(Expr::col((CommandsTable, CommandsColumn::Command)).eq(command))
                        .add(Expr::col((CommandAliasTable, CommandAliasColumn::Alias)).eq(command)),
                )
                .group_by_col((CommandsTable, CommandsColumn::Id)),
        )
        .await
    }

    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<Self>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .columns(CommandModel::columns())
                .from(CommandsTable)
                .and_where(Expr::col(CommandsColumn::Id).eq(id)),
        )
        .await
    }

    pub async fn all(db: &DbPool) -> DbResult<Vec<Self>> {
        sql_query_all(
            db,
            Query::select()
                .columns(CommandModel::columns())
                .from(CommandsTable)
                .order_by_columns([
                    (CommandsColumn::Order, Order::Asc),
                    (CommandsColumn::CreatedAt, Order::Desc),
                ]),
        )
        .await
    }

    pub async fn update(&mut self, db: &DbPool, data: UpdateCommand) -> anyhow::Result<()> {
        let mut update = Query::update();
        update
            .table(CommandsTable)
            .and_where(Expr::col(CommandsColumn::Id).eq(self.id));

        if let Some(enabled) = data.enabled {
            self.enabled = enabled;
            update.value(CommandsColumn::Enabled, Expr::value(enabled));
        }

        if let Some(name) = data.name {
            self.name = name.clone();
            update.value(CommandsColumn::Name, Expr::value(name));
        }

        if let Some(command) = data.command {
            self.command = command.clone();
            update.value(CommandsColumn::Command, Expr::value(command));
        }

        if let Some(config) = data.config {
            self.config = config;

            let config_value = serde_json::to_value(&self.config)?;
            update.value(CommandsColumn::Config, Expr::value(config_value));
        }

        if let Some(order) = data.order {
            self.order = order;
            update.value(CommandsColumn::Order, Expr::value(order));
        }

        sql_exec(db, &update).await?;

        if let Some(aliases) = data.aliases {
            CommandAliasModel::set_aliases(db, self.id, aliases).await?;
        }

        Ok(())
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let mut case = CaseStatement::new()
                // Use the current column value when not specified
                .finally(Expr::col(CommandsColumn::Order));

            // Add case for all updated values
            for order in order_chunk {
                case = case.case(
                    Expr::col(CommandsColumn::Id).eq(order.id),
                    Expr::value(order.order),
                );
            }

            sql_exec(
                db,
                Query::update()
                    .table(CommandsTable)
                    .value(CommandsColumn::Order, case),
            )
            .await?
        }

        Ok(())
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandsTable)
                .and_where(Expr::col(CommandsColumn::Id).eq(self.id)),
        )
        .await
    }
}
