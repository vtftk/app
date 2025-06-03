use super::shared::ExecutionsQuery;
use crate::{
    database::{DbErr, DbPool, DbResult},
    events::TwitchEventUser,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventExecutionModel {
    pub id: Uuid,
    pub event_id: Uuid,
    #[sqlx(json)]
    pub metadata: EventExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventExecutionMetadata {
    /// User who triggered the event
    pub user: Option<TwitchEventUser>,

    /// Catchall for any other metadata
    #[serde(flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub data: Vec<(String, serde_json::Value)>,
}

#[derive(Debug)]
pub struct CreateEventExecution {
    pub event_id: Uuid,
    pub metadata: EventExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

impl EventExecutionModel {
    /// Create a execution for a specific event
    pub async fn create(db: &DbPool, create: CreateEventExecution) -> DbResult<()> {
        let id = Uuid::new_v4();
        let metadata_value =
            serde_json::to_value(&create.metadata).map_err(|err| DbErr::Encode(err.into()))?;

        sqlx::query(
            r#"INSERT INTO "event_executions" ("id", "event_id", "metadata", "created_at")
            VALUES (?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(create.event_id)
        .bind(metadata_value)
        .bind(create.created_at)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Find the most recent execution of an event, with an offset
    /// to get the nth recent execution
    pub async fn last(
        db: &DbPool,
        event_id: Uuid,
        offset: u64,
    ) -> DbResult<Option<EventExecutionModel>> {
        sqlx::query_as(
            r#"SELECT * FROM "event_executions" 
            WHERE "event_id" = ? 
            ORDER BY "created_at" DESC OFFSET ? LIMIT 1"#,
        )
        .bind(event_id)
        .bind(offset as i64)
        .fetch_optional(db)
        .await
    }

    /// Query the executions for a specific event
    pub async fn query(
        db: &DbPool,
        event_id: Uuid,
        input: ExecutionsQuery,
    ) -> DbResult<Vec<EventExecutionModel>> {
        let condition = std::iter::once(r#""event_id" = ?"#)
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
            r#"SELECT * FROM "event_executions" WHERE {condition} {offset} 
            ORDER BY "created_at" DESC"#
        );

        let mut query = sqlx::query_as(&sql)
            // Bind event ID
            .bind(event_id);

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

    /// Deletes all executions that happened before the provided `start_time`.
    /// Used to clean out old executions
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "event_executions" WHERE "created_at" < ?"#)
            .bind(start_date)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Deletes a collection of specific executions by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders = std::iter::repeat_n('?', ids.len()).join(",");
        let sql = format!(r#"DELETE FROM "event_executions" WHERE "id" IN ({placeholders})"#);
        let mut query = sqlx::query(&sql);

        for id in ids {
            query = query.bind(id);
        }

        query.execute(db).await?;
        Ok(())
    }

    /// Estimate the size of all execution metadata in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        let result: (u32,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(LENGTH("metadata")), 0) FROM "event_executions""#,
        )
        .fetch_one(db)
        .await?;
        Ok(result.0)
    }
}
