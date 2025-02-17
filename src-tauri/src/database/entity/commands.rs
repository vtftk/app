use chrono::{DateTime, Utc};
use sea_query::{
    Alias, CaseStatement, Condition, Expr, Func, IdenStatic, JoinType, Order, Query,
    SqliteQueryBuilder,
};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    database::{DbPool, DbResult},
    events::TwitchEventUser,
};

use super::shared::{
    ExecutionsQuery, LoggingLevelDb, LogsQuery, MinimumRequireRole, UpdateOrdering,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
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

#[derive(Debug)]
pub struct CreateCommandLog {
    pub command_id: Uuid,
    pub level: LoggingLevelDb,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CommandLogsModel {
    /// Unique ID of the log
    pub id: Uuid,
    /// ID of the command
    pub command_id: Uuid,
    /// Level of the log
    pub level: LoggingLevelDb,
    /// Logging message
    pub message: String,
    /// Creation time of the event
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateCommandExecution {
    pub command_id: Uuid,
    pub metadata: CommandExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CommandExecutionModel {
    /// Unique ID for the event
    pub id: Uuid,
    pub command_id: Uuid,
    #[sqlx(json)]
    pub metadata: CommandExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandExecutionMetadata {
    /// User who triggered the event
    pub user: Option<TwitchEventUser>,

    /// Catchall for any other metadata
    #[serde(flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub data: Vec<(String, serde_json::Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandOutcome {
    Template { message: String },
    Script { script: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

        model.set_aliases(db, create.aliases).await?;

        Ok(model)
    }

    /// Find commands by the actual command trigger word
    /// and only commands that are enabled
    pub async fn get_by_command(db: &DbPool, command: &str) -> DbResult<Vec<CommandModel>> {
        let (sql, values) = Query::select()
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
                    .add(Expr::expr(Func::lower(Expr::col(CommandsColumn::Command))).eq(command))
                    .add(Expr::expr(Func::lower(Expr::col(CommandAliasColumn::Alias))).eq(command)),
            )
            .and_where(Expr::col(CommandsColumn::Enabled).eq(true))
            .group_by_col(CommandsColumn::Id)
            .build_sqlx(SqliteQueryBuilder);

        let results = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results)
    }

    /// Find the most recent execution of this command
    pub async fn last_execution(
        &self,
        db: &DbPool,
        offset: u64,
    ) -> DbResult<Option<CommandExecutionModel>> {
        let (sql, values) = Query::select()
            .from(CommandExecutionsTable)
            .columns([
                CommandExecutionsColumn::Id,
                CommandExecutionsColumn::CommandId,
                CommandExecutionsColumn::Metadata,
                CommandExecutionsColumn::CreatedAt,
            ])
            .and_where(Expr::col(CommandExecutionsColumn::CommandId).eq(self.id))
            .offset(offset)
            .order_by(CommandExecutionsColumn::CreatedAt, Order::Desc)
            .build_sqlx(SqliteQueryBuilder);

        let value: Option<CommandExecutionModel> =
            sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(value)
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(CommandsTable)
            .and_where(Expr::col(CommandsColumn::Id).eq(self.id))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<Self>> {
        let (sql, values) = Query::select()
            .columns(CommandModel::columns())
            .from(CommandsTable)
            .and_where(Expr::col(CommandsColumn::Id).eq(id))
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(result)
    }

    pub async fn get_by_id_with_aliases(
        db: &DbPool,
        id: Uuid,
    ) -> DbResult<Option<CommandWithAliases>> {
        let command = match Self::get_by_id(db, id).await? {
            Some(value) => value,
            None => return Ok(None),
        };
        let aliases = command.get_aliases(db).await?;

        Ok(Some(CommandWithAliases { command, aliases }))
    }

    pub async fn all(db: &DbPool) -> DbResult<Vec<Self>> {
        let (sql, values) = Query::select()
            .columns(CommandModel::columns())
            .from(CommandsTable)
            .order_by_columns([
                (CommandsColumn::Order, Order::Asc),
                (CommandsColumn::CreatedAt, Order::Desc),
            ])
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(result)
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

        if let Some(aliases) = data.aliases {
            self.set_aliases(db, aliases).await?;
        }

        Ok(())
    }

    pub async fn get_logs(&self, db: &DbPool, query: LogsQuery) -> DbResult<Vec<CommandLogsModel>> {
        let mut select = Query::select();
        select
            .from(CommandLogsTable)
            .columns([
                CommandLogsColumn::Id,
                CommandLogsColumn::CommandId,
                CommandLogsColumn::Level,
                CommandLogsColumn::Message,
                CommandLogsColumn::CreatedAt,
            ])
            .and_where(Expr::col(CommandLogsColumn::CommandId).eq(self.id))
            .order_by(CommandLogsColumn::CreatedAt, Order::Desc);

        if let Some(level) = query.level {
            select.and_where(Expr::col(CommandLogsColumn::Level).eq(level as i32));
        }

        if let Some(start_date) = query.start_date {
            select.and_where(Expr::col(CommandLogsColumn::CreatedAt).gt(start_date));
        }

        if let Some(end_date) = query.end_date {
            select.and_where(Expr::col(CommandLogsColumn::CreatedAt).lt(end_date));
        }

        if let Some(offset) = query.offset {
            select.offset(offset);
        }

        if let Some(limit) = query.limit {
            select.limit(limit);
        }

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        let results = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results)
    }

    pub async fn get_aliases(&self, db: &DbPool) -> DbResult<Vec<String>> {
        #[derive(FromRow)]
        struct Alias {
            alias: String,
        }

        let (sql, values) = Query::select()
            .columns([CommandAliasColumn::Alias])
            .from(CommandAliasTable)
            .and_where(Expr::col(CommandAliasColumn::CommandId).eq(self.id))
            .order_by(CommandAliasColumn::Order, Order::Asc)
            .build_sqlx(SqliteQueryBuilder);

        let results: Vec<Alias> = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results.into_iter().map(|value| value.alias).collect())
    }

    pub async fn set_aliases(&self, db: &DbPool, aliases: Vec<String>) -> DbResult<()> {
        // Delete all command aliases for the command
        {
            let (sql, values) = Query::delete()
                .from_table(CommandAliasTable)
                .and_where(Expr::col(CommandAliasColumn::CommandId).eq(self.id))
                .build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(db).await?;
        }

        // Insert new aliases
        {
            let (sql, values) = Query::insert()
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
                        self.id.into(),
                        alias.into(),
                        (index as u32).into(),
                    ]
                }))
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(db).await?;
        }

        Ok(())
    }

    pub async fn get_executions(
        &self,
        db: &DbPool,
        query: ExecutionsQuery,
    ) -> DbResult<Vec<CommandExecutionModel>> {
        let mut select = Query::select();
        select
            .from(CommandExecutionsTable)
            .columns([
                CommandExecutionsColumn::Id,
                CommandExecutionsColumn::CommandId,
                CommandExecutionsColumn::Metadata,
                CommandExecutionsColumn::CreatedAt,
            ])
            .and_where(Expr::col(CommandExecutionsColumn::CommandId).eq(self.id))
            .order_by(CommandExecutionsColumn::CreatedAt, Order::Desc);

        if let Some(start_date) = query.start_date {
            select.and_where(Expr::col(CommandExecutionsColumn::CreatedAt).gt(start_date));
        }

        if let Some(end_date) = query.end_date {
            select.and_where(Expr::col(CommandExecutionsColumn::CreatedAt).lt(end_date));
        }

        if let Some(offset) = query.offset {
            select.offset(offset);
        }

        if let Some(limit) = query.limit {
            select.limit(limit);
        }

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        let results = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results)
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

            let (sql, values) = Query::update()
                .table(CommandsTable)
                .value(CommandsColumn::Order, case)
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(db).await?;
        }

        Ok(())
    }

    pub async fn create_log(db: &DbPool, create: CreateCommandLog) -> DbResult<()> {
        let id = Uuid::new_v4();

        let (sql, values) = Query::insert()
            .into_table(CommandLogsTable)
            .columns([
                CommandLogsColumn::Id,
                CommandLogsColumn::CommandId,
                CommandLogsColumn::Level,
                CommandLogsColumn::Message,
                CommandLogsColumn::CreatedAt,
            ])
            .values_panic([
                id.into(),
                create.command_id.into(),
                (create.level as i32).into(),
                create.message.to_string().into(),
                create.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(())
    }

    pub async fn delete_many_logs(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(CommandLogsTable)
            .and_where(Expr::col(CommandLogsColumn::Id).is_in(ids.iter().copied()))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn delete_logs_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(CommandLogsTable)
            .and_where(Expr::col(CommandLogsColumn::CreatedAt).lt(start_date))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn get_logs_estimate_size(db: &DbPool) -> DbResult<u32> {
        #[derive(Default, FromRow)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let (sql, values) = Query::select()
            .from(CommandLogsTable)
            .expr_as(
                Func::sum(Func::char_length(Expr::col(CommandLogsColumn::Message))),
                Alias::new("total_message_length"),
            )
            .build_sqlx(SqliteQueryBuilder);

        let result: PartialModel = sqlx::query_as_with(&sql, values).fetch_one(db).await?;
        Ok(result.total_message_length.unwrap_or_default())
    }

    pub async fn create_execution(
        db: &DbPool,
        create: CreateCommandExecution,
    ) -> anyhow::Result<CommandExecutionModel> {
        let id = Uuid::new_v4();
        let model = CommandExecutionModel {
            id,
            command_id: create.command_id,
            metadata: create.metadata,
            created_at: create.created_at,
        };

        let metadata_value = serde_json::to_value(&model.metadata)?;

        let (sql, values) = Query::insert()
            .into_table(CommandExecutionsTable)
            .columns([
                CommandExecutionsColumn::Id,
                CommandExecutionsColumn::CommandId,
                CommandExecutionsColumn::Metadata,
                CommandExecutionsColumn::CreatedAt,
            ])
            .values_panic([
                model.id.into(),
                model.command_id.into(),
                metadata_value.into(),
                model.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(model)
    }

    pub async fn delete_executions_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(CommandExecutionsTable)
            .and_where(Expr::col(CommandExecutionsColumn::CreatedAt).lt(start_date))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn delete_many_executions(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(CommandExecutionsTable)
            .and_where(Expr::col(CommandExecutionsColumn::Id).is_in(ids.iter().copied()))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn get_executions_estimate_size(db: &DbPool) -> DbResult<u32> {
        #[derive(Default, FromRow)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let (sql, values) = Query::select()
            .from(CommandExecutionsTable)
            .expr_as(
                Func::sum(Func::char_length(Expr::col(
                    CommandExecutionsColumn::Metadata,
                ))),
                Alias::new("total_message_length"),
            )
            .build_sqlx(SqliteQueryBuilder);

        let result: PartialModel = sqlx::query_as_with(&sql, values).fetch_one(db).await?;
        Ok(result.total_message_length.unwrap_or_default())
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
