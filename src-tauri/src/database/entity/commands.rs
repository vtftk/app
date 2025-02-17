use super::{
    command_alias::{CommandAliasColumn, CommandAliasModel, CommandAliasTable},
    shared::{MinimumRequireRole, UpdateOrdering},
};
use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_maybe_one},
    DbPool, DbResult,
};
use chrono::{DateTime, Utc};
use sea_query::{
    Alias, CaseStatement, Condition, Expr, Func, IdenStatic, JoinType, Order, Query,
    SqliteQueryBuilder,
};
use sea_query_binder::SqlxBinder;
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
    /// The outcome of the command
    #[sqlx(json)]
    pub outcome: CommandOutcome,
    /// Cooldown between each trigger of the command
    #[sqlx(json)]
    pub cooldown: CommandCooldown,
    /// Minimum required role to trigger the command
    pub require_role: MinimumRequireRole,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
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
    pub outcome: CommandOutcome,
    pub cooldown: CommandCooldown,
    pub require_role: MinimumRequireRole,
    pub aliases: Vec<String>,
}

#[derive(Default, Deserialize)]
pub struct UpdateCommand {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub command: Option<String>,
    pub outcome: Option<CommandOutcome>,
    pub cooldown: Option<CommandCooldown>,
    pub require_role: Option<MinimumRequireRole>,
    pub order: Option<u32>,
    pub aliases: Option<Vec<String>>,
}

impl CommandModel {
    pub fn columns() -> [CommandsColumn; 9] {
        [
            CommandsColumn::Id,
            CommandsColumn::Enabled,
            CommandsColumn::Name,
            CommandsColumn::Command,
            CommandsColumn::Outcome,
            CommandsColumn::Cooldown,
            CommandsColumn::RequireRole,
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
            command: create.command,
            outcome: create.outcome,
            cooldown: create.cooldown,
            require_role: create.require_role,
            order: 0,
            created_at: Utc::now(),
        };

        let cooldown_value = serde_json::to_value(&model.cooldown)?;
        let outcome_value = serde_json::to_value(&model.outcome)?;

        let (sql, values) = Query::insert()
            .into_table(CommandsTable)
            .columns(CommandModel::columns())
            .values_panic([
                model.id.into(),
                model.enabled.into(),
                model.name.clone().into(),
                model.command.to_string().into(),
                outcome_value.into(),
                cooldown_value.into(),
                model.require_role.to_string().into(),
                model.order.into(),
                model.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

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
                .columns(CommandModel::columns())
                .join_as(
                    JoinType::LeftJoin,
                    CommandAliasTable,
                    Alias::new("alias"),
                    Expr::col((CommandsTable, CommandsColumn::Id))
                        .equals((CommandAliasTable, CommandAliasColumn::CommandId)),
                )
                .cond_where(
                    Condition::any()
                        .add(
                            Expr::expr(Func::lower(Expr::col(CommandsColumn::Command))).eq(command),
                        )
                        .add(
                            Expr::expr(Func::lower(Expr::col(CommandAliasColumn::Alias)))
                                .eq(command),
                        ),
                )
                .and_where(Expr::col(CommandsColumn::Enabled).eq(true))
                .group_by_col(CommandsColumn::Id),
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
        update.table(CommandsTable);

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

        if let Some(outcome) = data.outcome {
            self.outcome = outcome;

            let outcome_value = serde_json::to_value(&self.outcome)?;
            update.value(CommandsColumn::Outcome, Expr::value(outcome_value));
        }

        if let Some(cooldown) = data.cooldown {
            self.cooldown = cooldown;

            let cooldown_value = serde_json::to_value(&self.cooldown)?;
            update.value(CommandsColumn::Cooldown, Expr::value(cooldown_value));
        }

        if let Some(require_role) = data.require_role {
            self.require_role = require_role;
            update.value(
                CommandsColumn::RequireRole,
                Expr::value(require_role.to_string()),
            );
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
