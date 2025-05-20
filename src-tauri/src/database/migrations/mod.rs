use super::{DbPool, DbResult};
use anyhow::Context;
use chrono::{DateTime, Utc};
use log::warn;
use sqlx::prelude::FromRow;

mod m20241208_060123_create_items_table;
mod m20241208_060138_create_events_table;
mod m20241208_060144_create_sounds_table;
mod m20241208_060200_create_commands_table;
mod m20241208_060230_create_model_data_table;
mod m20241208_063859_create_items_sounds_junction_table;
mod m20241210_082256_create_event_executions_table;
mod m20241210_082316_create_command_executions_table;
mod m20241211_102725_seed_defaults;
mod m20241212_114700_create_key_value_table;
mod m20241214_080902_create_command_logs_table;
mod m20241227_110419_create_event_logs_table;
mod m20250104_071851_create_app_data_table;
mod m20250124_082703_create_chat_history_table;
mod m20250209_101257_create_command_aliases_table;
mod m20250216_140137_create_secrets_table;

fn migrations() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(m20241208_060123_create_items_table::ItemsMigration),
        Box::new(m20241208_060138_create_events_table::EventsMigration),
        Box::new(m20241208_060144_create_sounds_table::SoundsMigration),
        Box::new(m20241208_060200_create_commands_table::CommandsMigration),
        Box::new(m20241208_060230_create_model_data_table::ModelDataMigration),
        Box::new(m20241208_063859_create_items_sounds_junction_table::ItemsSoundsMigration),
        Box::new(m20241210_082256_create_event_executions_table::EventExecutionsMigration),
        Box::new(m20241210_082316_create_command_executions_table::CommandExecutionsMigration),
        Box::new(m20241211_102725_seed_defaults::SeedDefaultsMigration),
        Box::new(m20241212_114700_create_key_value_table::KeyValueMigration),
        Box::new(m20241214_080902_create_command_logs_table::CommandLogsMigration),
        Box::new(m20241227_110419_create_event_logs_table::EventLogsMigration),
        Box::new(m20250104_071851_create_app_data_table::AppDataMigration),
        Box::new(m20250124_082703_create_chat_history_table::ChatHistoryMigration),
        Box::new(m20250209_101257_create_command_aliases_table::CommandAliasesMigration),
        Box::new(m20250216_140137_create_secrets_table::SecretsMigration),
    ]
}

#[async_trait::async_trait]
pub trait Migration {
    fn name(&self) -> &str;

    async fn up(&self, db: &DbPool) -> anyhow::Result<()>;
}

#[derive(FromRow)]
struct AppliedMigration {
    name: String,
    #[allow(unused)]
    applied_at: DateTime<Utc>,
}

pub async fn migrate(db: &DbPool) -> anyhow::Result<()> {
    create_migrations_table(db)
        .await
        .context("failed to create migrations table")?;

    let migrations = migrations();
    let mut applied = get_applied_migrations(db)
        .await
        .context("failed to get applied migrations")?;
    let mut migration_names = Vec::new();

    for migration in &migrations {
        let name = migration.name();
        migration_names.push(name.to_string());

        // Migration already applied
        if applied.iter().any(|applied| applied.name.eq(name)) {
            continue;
        }

        // Apply migration
        migration
            .up(db)
            .await
            .with_context(|| format!("failed to apply migration \"{name}\""))?;

        // Store applied migration
        let applied_at = Utc::now();
        let migration = create_applied_migration(db, name.to_string(), applied_at)
            .await
            .with_context(|| format!("failed to store applied migration \"{name}\""))?;

        applied.push(migration);
    }

    // Check if a migration was applied but is not known locally (warning)
    for applied in applied {
        if !migration_names.contains(&applied.name) {
            warn!(
                "database has migration applied that is not known locally: \"{}\"",
                &applied.name
            );
        }
    }

    Ok(())
}

async fn create_migrations_table(db: &DbPool) -> DbResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "migrations" (
            "name"	VARCHAR NOT NULL PRIMARY KEY,
            "applied_at"	datetime_text NOT NULL,
        );
    "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn get_applied_migrations(db: &DbPool) -> DbResult<Vec<AppliedMigration>> {
    let result: Vec<AppliedMigration> = sqlx::query_as(r#"SELECT * FROM "migrations""#)
        .fetch_all(db)
        .await?;
    Ok(result)
}

async fn create_applied_migration(
    db: &DbPool,
    name: String,
    applied_at: DateTime<Utc>,
) -> DbResult<AppliedMigration> {
    sqlx::query(r#"INSERT INTO "migrations" ("name", "applied_at") VALUES (?, ?)"#)
        .bind(name.as_str())
        .bind(applied_at)
        .execute(db)
        .await?;

    let model = AppliedMigration { name, applied_at };
    Ok(model)
}
