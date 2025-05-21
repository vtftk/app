use anyhow::Context;
use chrono::{Days, Utc};

use entity::{
    app_data::AppDataModel, chat_history::ChatHistoryModel,
    command_execution::CommandExecutionModel, command_log::CommandLogsModel,
    event_execution::EventExecutionModel, event_log::EventLogsModel,
};
use log::error;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{path::PathBuf, str::FromStr};
use tokio::fs::{create_dir_all, File};

pub mod entity;
mod migrations;

pub type DbPool = SqlitePool;
pub type DbErr = sqlx::Error;
pub type DbResult<T> = Result<T, DbErr>;

/// Connects to the SQLite database at the provided path, creating a
/// new database file if none exist
pub async fn connect_database(path: PathBuf) -> anyhow::Result<DbPool> {
    if !path.exists() {
        let parent = path.parent().context("database path invalid")?;
        create_dir_all(parent)
            .await
            .context("create database path")?;

        File::create(&path).await?;
    }

    let path = path.to_str().context("invalid db path")?;
    let path = format!("sqlite://{path}");

    let options = SqliteConnectOptions::from_str(&path).context("failed to parse connection")?;
    let db = SqlitePool::connect_with(options)
        .await
        .context("failed to connect")?;

    setup_database(&db).await.context("failed to setup")?;

    Ok(db)
}

#[cfg(test)]
pub async fn mock_database() -> DbPool {
    let db = SqlitePool::connect_with(SqliteConnectOptions::from_str("sqlite::memory:").unwrap())
        .await
        .unwrap();

    setup_database(&db).await.unwrap();
    db
}

pub async fn setup_database(db: &DbPool) -> anyhow::Result<()> {
    if let Err(cause) = migrations::migrate(db).await {
        error!("failed to migrate database: {cause:?}");
        return Err(cause.context("failed to migrate database"));
    }
    Ok(())
}

pub async fn clean_old_data(db: DbPool) -> anyhow::Result<()> {
    let main_config = AppDataModel::get_main_config(&db).await?;

    let now = Utc::now();

    // Clean logs
    if main_config.clean_logs {
        let clean_date = now
            .checked_sub_days(Days::new(main_config.clean_logs_days))
            .context("system time is incorrect")?;

        EventLogsModel::delete_before(&db, clean_date).await?;
        CommandLogsModel::delete_before(&db, clean_date).await?;
    }

    // Clean executions
    if main_config.clean_executions {
        let clean_date = now
            .checked_sub_days(Days::new(main_config.clean_executions_days))
            .context("system time is incorrect")?;

        EventExecutionModel::delete_before(&db, clean_date).await?;
        CommandExecutionModel::delete_before(&db, clean_date).await?;
    }

    // Clean chat history
    if main_config.clean_chat_history {
        let clean_date = now
            .checked_sub_days(Days::new(main_config.clean_chat_history_days))
            .context("system time is incorrect")?;

        ChatHistoryModel::delete_before(&db, clean_date).await?;
    }

    Ok(())
}
