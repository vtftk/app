use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{
    entity::{
        command_alias::CommandAliasModel,
        commands::{CommandConfig, CommandModel, CreateCommand},
        events::{CreateEvent, EventConfig, EventModel},
    },
    DbPool,
};

#[derive(Serialize, Deserialize)]
pub struct ExportedCommandModel {
    pub enabled: bool,
    pub name: String,
    pub command: String,
    pub config: CommandConfig,
    pub aliases: Vec<String>,
}

/// Export a collection of commands by ID
pub async fn export_commands(
    db: &DbPool,
    ids: &[Uuid],
) -> anyhow::Result<Vec<ExportedCommandModel>> {
    let mut exported = Vec::new();

    for id in ids {
        let command = match CommandModel::get_by_id(db, *id).await? {
            Some(value) => value,
            None => continue,
        };
        let aliases = CommandAliasModel::get_aliases(db, *id).await?;

        exported.push(ExportedCommandModel {
            enabled: command.enabled,
            name: command.name,
            command: command.command,
            config: command.config,
            aliases,
        });
    }

    Ok(exported)
}

/// Import a collection of commands
pub async fn import_commands(
    db: &DbPool,
    commands: Vec<ExportedCommandModel>,
) -> anyhow::Result<()> {
    for command in commands {
        CommandModel::create(
            db,
            CreateCommand {
                enabled: command.enabled,
                name: command.name,
                command: command.command,
                config: command.config,
                aliases: command.aliases,
            },
        )
        .await?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ExportedEventModel {
    pub enabled: bool,
    pub name: String,
    pub config: EventConfig,
}

/// Export a collection of commands by ID
pub async fn export_events(db: &DbPool, ids: &[Uuid]) -> anyhow::Result<Vec<ExportedEventModel>> {
    let command = EventModel::get_by_ids(db, ids).await?;
    Ok(command
        .into_iter()
        .map(|command| ExportedEventModel {
            enabled: command.enabled,
            name: command.name,
            config: command.config,
        })
        .collect())
}

/// Import a collection of events
pub async fn import_events(db: &DbPool, events: Vec<ExportedEventModel>) -> anyhow::Result<()> {
    for event in events {
        EventModel::create(
            db,
            CreateEvent {
                enabled: event.enabled,
                name: event.name,
                config: event.config,
            },
        )
        .await?;
    }

    Ok(())
}
