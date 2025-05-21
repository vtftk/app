use super::{DbPool, DbResult};
use anyhow::Context;
use chrono::{DateTime, Utc};
use log::warn;
use sqlx::prelude::FromRow;

#[rustfmt::skip]
const MIGRATIONS: &[(&str, &str)]= &[
    ("m20241208_060123_create_items_table", include_str!("sql/m20241208_060123_create_items_table.sql")),
    ("m20241208_060138_create_events_table", include_str!("sql/m20241208_060138_create_events_table.sql")),
    ("m20241208_060144_create_sounds_table", include_str!("sql/m20241208_060144_create_sounds_table.sql")),
    ("m20241208_060200_create_commands_table", include_str!("sql/m20241208_060200_create_commands_table.sql")),
    ("m20241208_060230_create_model_data_table", include_str!("sql/m20241208_060230_create_model_data_table.sql")),
    ("m20241210_082256_create_event_executions_table", include_str!("sql/m20241210_082256_create_event_executions_table.sql")),
    ("m20241210_082316_create_command_executions_table", include_str!("sql/m20241210_082316_create_command_executions_table.sql")),
    ("m20241211_102725_seed_defaults", include_str!("sql/m20241211_102725_seed_defaults.sql")),
    ("m20241212_114700_create_key_value_table", include_str!("sql/m20241212_114700_create_key_value_table.sql")),
    ("m20241214_080902_create_command_logs_table", include_str!("sql/m20241214_080902_create_command_logs_table.sql")),
    ("m20241227_110419_create_event_logs_table", include_str!("sql/m20241227_110419_create_event_logs_table.sql")),
    ("m20250104_071851_create_app_data_table", include_str!("sql/m20250104_071851_create_app_data_table.sql")),
    ("m20250124_082703_create_chat_history_table", include_str!("sql/m20250124_082703_create_chat_history_table.sql")),
    ("m20250209_101257_create_command_aliases_table", include_str!("sql/m20250209_101257_create_command_aliases_table.sql")),
    ("m20250216_140137_create_secrets_table", include_str!("sql/m20250216_140137_create_secrets_table.sql")),
];

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

    let mut applied = get_applied_migrations(db)
        .await
        .context("failed to get applied migrations")?;
    let mut migration_names = Vec::new();

    for (name, sql) in MIGRATIONS {
        migration_names.push(name.to_string());

        // Migration already applied
        if applied.iter().any(|applied| applied.name.eq(name)) {
            continue;
        }

        // Apply migration
        sqlx::raw_sql(sql)
            .execute(db)
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
            "applied_at"	datetime_text NOT NULL
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
