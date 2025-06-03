use super::shared::{LoggingLevelDb, LogsQuery};
use crate::database::{DbPool, DbResult};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventLogsModel {
    /// Unique ID of the log
    pub id: Uuid,
    /// ID of the event
    pub event_id: Uuid,
    /// Level of the log
    pub level: LoggingLevelDb,
    /// Logging message
    pub message: String,
    /// Creation time of the event
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateEventLog {
    pub event_id: Uuid,
    pub level: LoggingLevelDb,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl EventLogsModel {
    /// Create a new log for a specific event
    pub async fn create(db: &DbPool, create: CreateEventLog) -> DbResult<()> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO "event_logs" ("id", "event_id", "level", "message", "created_at")
            VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(create.event_id)
        .bind(create.level)
        .bind(create.message)
        .bind(create.created_at)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Query the logs for a specific event
    pub async fn query(
        db: &DbPool,
        event_id: Uuid,
        input: LogsQuery,
    ) -> DbResult<Vec<EventLogsModel>> {
        let condition = std::iter::once(r#""event_id" = ?"#)
            // Optional level filter
            .chain(input.level.map(|_| r#"WHERE "level" = ?"#))
            // Filter from start date
            .chain(input.start_date.map(|_| r#"WHERE "created_at" >= ?"#))
            // Filter from end date
            .chain(input.end_date.map(|_| r#"WHERE "created_at" <= ?"#))
            // Join into condition
            .join(" OR ");

        let offset = if input.offset.is_some() && input.limit.is_some() {
            "OFFSET ? LIMIT ?"
        } else {
            ""
        };

        let sql = format!(
            r#"SELECT * FROM "event_logs" WHERE {condition} {offset} 
            ORDER BY "created_at" DESC"#
        );

        let mut query = sqlx::query_as(&sql)
            // Bind event ID
            .bind(event_id);

        if let Some(level) = input.level {
            query = query.bind(level as i32)
        }

        if let Some(start_date) = input.start_date {
            query = query.bind(start_date)
        }

        if let Some(end_date) = input.end_date {
            query = query.bind(end_date)
        }

        if let (Some(offset), Some(limit)) = (input.offset, input.limit) {
            query = query.bind(offset as i64).bind(limit as i64)
        }

        query.fetch_all(db).await
    }

    /// Deletes all logs that happened before the provided `start_time`.
    /// Used to clean out old logs
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "event_logs" WHERE "created_at" < ?"#)
            .bind(start_date)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Deletes a collection of specific logs by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders = std::iter::repeat_n('?', ids.len()).join(",");
        let sql = format!(r#"DELETE FROM "event_logs" WHERE "id" IN ({placeholders})"#);
        let mut query = sqlx::query(&sql);

        for id in ids {
            query = query.bind(id);
        }

        query.execute(db).await?;
        Ok(())
    }

    /// Estimate the size of all log messages in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        let result: (u32,) =
            sqlx::query_as(r#"SELECT COALESCE(SUM(LENGTH("message")), 0) FROM "event_logs""#)
                .fetch_one(db)
                .await?;
        Ok(result.0)
    }
}
