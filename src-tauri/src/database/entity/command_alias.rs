use super::commands::CommandModel;
use crate::database::{DbPool, DbResult};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
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

impl CommandAliasModel {
    /// Get all aliases for a specific command
    pub async fn get_aliases(db: &DbPool, command_id: Uuid) -> DbResult<Vec<String>> {
        let results: Vec<(String,)> = sqlx::query_as(
            r#"SELECT "alias" FROM "command_alias" WHERE "command_id" = ? ORDER BY "order" ASC"#,
        )
        .bind(command_id)
        .fetch_all(db)
        .await?;

        Ok(results.into_iter().map(|(alias,)| alias).collect())
    }

    async fn delete_aliases(db: &DbPool, command_id: Uuid) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "command_alias" WHERE "command_id" = ?"#)
            .bind(command_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Sets all the aliases for a specific command
    /// (Removes the previous set and creates the new one)
    pub async fn set_aliases(db: &DbPool, command_id: Uuid, aliases: Vec<String>) -> DbResult<()> {
        // Delete all command aliases for the command
        Self::delete_aliases(db, command_id).await?;

        // Don't try and insert if theres no data
        if aliases.is_empty() {
            return Ok(());
        }

        // Generate the placeholders required to insert values
        let values_sets = std::iter::repeat_n("(?,?,?,?)", aliases.len()).join(",");

        let sql = format!(
            r#"INSERT INTO "command_alias" ("id", "command_id", "alias", "order")
            VALUES {values_sets}
            "#
        );

        let mut query = sqlx::query(&sql);

        for (index, alias) in aliases.into_iter().enumerate() {
            query = query
                .bind(Uuid::new_v4())
                .bind(command_id)
                .bind(alias)
                .bind(index as i64);
        }

        query.execute(db).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::CommandAliasModel;
    use crate::database::{
        entity::{
            commands::{CommandConfig, CommandModel, CommandOutcome, CreateCommand},
            shared::MinimumRequireRole,
        },
        mock_database,
    };

    /// Tests that the aliases can be set initially
    #[tokio::test]
    async fn test_set_aliases_initial() {
        let db = mock_database().await;
        let command = CommandModel::create(
            &db,
            CreateCommand {
                enabled: true,
                name: "test".to_string(),
                command: "!test".to_string(),
                config: CommandConfig {
                    outcome: CommandOutcome::Template {
                        message: "test".to_string(),
                    },
                    cooldown: Default::default(),
                    require_role: MinimumRequireRole::None,
                },
                aliases: vec![],
            },
        )
        .await
        .unwrap();

        CommandAliasModel::set_aliases(
            &db,
            command.id,
            vec!["!test1".to_string(), "!test2".to_string()],
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_set_aliases_update() {
        let db = mock_database().await;
        let command = CommandModel::create(
            &db,
            CreateCommand {
                enabled: true,
                name: "test".to_string(),
                command: "!test".to_string(),
                config: CommandConfig {
                    outcome: CommandOutcome::Template {
                        message: "test".to_string(),
                    },
                    cooldown: Default::default(),
                    require_role: MinimumRequireRole::None,
                },
                aliases: vec![],
            },
        )
        .await
        .unwrap();

        CommandAliasModel::set_aliases(
            &db,
            command.id,
            vec!["!test1".to_string(), "!test2".to_string()],
        )
        .await
        .unwrap();

        let aliases = CommandAliasModel::get_aliases(&db, command.id)
            .await
            .unwrap();

        assert_eq!(aliases, vec!["!test1".to_string(), "!test2".to_string()]);

        CommandAliasModel::set_aliases(
            &db,
            command.id,
            vec!["!test3".to_string(), "!test4".to_string()],
        )
        .await
        .unwrap();

        let aliases = CommandAliasModel::get_aliases(&db, command.id)
            .await
            .unwrap();

        assert_eq!(aliases, vec!["!test3".to_string(), "!test4".to_string()]);
    }

    #[tokio::test]
    async fn test_get_aliases() {
        let db = mock_database().await;
        let command = CommandModel::create(
            &db,
            CreateCommand {
                enabled: true,
                name: "test".to_string(),
                command: "!test".to_string(),
                config: CommandConfig {
                    outcome: CommandOutcome::Template {
                        message: "test".to_string(),
                    },
                    cooldown: Default::default(),
                    require_role: MinimumRequireRole::None,
                },
                aliases: vec![],
            },
        )
        .await
        .unwrap();

        let aliases = CommandAliasModel::get_aliases(&db, command.id)
            .await
            .unwrap();

        assert!(aliases.is_empty());

        CommandAliasModel::set_aliases(
            &db,
            command.id,
            vec!["!test3".to_string(), "!test4".to_string()],
        )
        .await
        .unwrap();

        let aliases = CommandAliasModel::get_aliases(&db, command.id)
            .await
            .unwrap();

        assert_eq!(aliases, vec!["!test3".to_string(), "!test4".to_string()]);
    }
}
