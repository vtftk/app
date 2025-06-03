use super::{
    command_alias::CommandAliasModel,
    shared::{MinimumRequireRole, UpdateOrdering},
};
use crate::database::{DbErr, DbPool, DbResult};
use chrono::{DateTime, Utc};
use itertools::Itertools;
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
    pub aliases: Option<Vec<String>>,
}

impl CommandModel {
    /// Create a new sound
    pub async fn create(db: &DbPool, create: CreateCommand) -> DbResult<CommandModel> {
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

        let config_value =
            serde_json::to_value(&model.config).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(
            r#"
            INSERT INTO "commands" ("id", "enabled", "name", "command", "config", "order", "created_at")
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.enabled)
        .bind(model.name.as_str())
        .bind(model.command.as_str())
        .bind(config_value)
        .bind(model.order)
        .bind(model.created_at)
        .execute(db)
        .await?;

        // Set the command aliases
        CommandAliasModel::set_aliases(db, id, create.aliases).await?;

        Ok(model)
    }

    /// Find commands by the actual command trigger word
    /// and only commands that are enabled
    pub async fn get_by_command(db: &DbPool, command: &str) -> DbResult<Vec<CommandModel>> {
        sqlx::query_as(
            r#"SELECT "c".* FROM "commands"
            LEFT JOIN "command_alias" ON "command_alias"."command_id" = "commands"."id"
            WHERE "commands"."enabled" = TRUE AND ("commands"."command" = ? OR "command_alias"."alias" = ?)
            GROUP BY "commands"."id"
        "#,
        )
        .bind(command)
        .bind(command)
        .fetch_all(db)
        .await
    }

    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<Self>> {
        sqlx::query_as(r#"SELECT * FROM "commands" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    pub async fn all(db: &DbPool) -> DbResult<Vec<Self>> {
        sqlx::query_as(r#"SELECT * FROM "commands" ORDER BY "order" ASC, "created_at" DESC"#)
            .fetch_all(db)
            .await
    }

    pub async fn update(&mut self, db: &DbPool, data: UpdateCommand) -> DbResult<()> {
        let enabled = data.enabled.unwrap_or(self.enabled);
        let name = data.name.unwrap_or_else(|| self.name.clone());
        let command = data.command.unwrap_or_else(|| self.command.clone());
        let config = data.config.unwrap_or_else(|| self.config.clone());
        let config_value =
            serde_json::to_value(&config).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(
            r#"UPDATE "commands" SET "enabled" = ?, "name" = ?, "command" = ?, "config" = ? WHERE "id" = ?"#,
        )
        .bind(enabled)
        .bind(name.as_str())
        .bind(command.as_str())
        .bind(config_value)
        .bind(self.id)
        .execute(db)
        .await?;

        self.enabled = enabled;
        self.name = name;
        self.command = command;
        self.config = config;

        if let Some(aliases) = data.aliases {
            CommandAliasModel::set_aliases(db, self.id, aliases).await?;
        }

        Ok(())
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let cases = std::iter::repeat_n("WHEN ? THEN ?", order_chunk.len()).join(" ");

            let sql = format!(
                r#"
                UPDATE "commands"
                SET "order" = CASE "id"
                    {cases}
                    ELSE "order"
                END
            "#
            );

            let mut query = sqlx::query(&sql);

            for order in order_chunk {
                query = query.bind(order.id).bind(order.order);
            }

            query.execute(db).await?;
        }

        Ok(())
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "commands" WHERE "id" = ?"#)
            .bind(self.id)
            .execute(db)
            .await?;

        Ok(())
    }
}
