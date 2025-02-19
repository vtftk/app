use chrono::{DateTime, Utc};
use sea_query::{Expr, Func, IdenStatic, Order, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_one_single},
    DbPool, DbResult,
};

use super::shared::{LoggingLevelDb, LogsQuery};

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "event_logs")]
pub struct EventLogsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventLogsColumn {
    Id,
    EventId,
    Level,
    Message,
    CreatedAt,
}

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

        sql_exec(
            db,
            Query::insert()
                .into_table(EventLogsTable)
                .columns([
                    EventLogsColumn::Id,
                    EventLogsColumn::EventId,
                    EventLogsColumn::Level,
                    EventLogsColumn::Message,
                    EventLogsColumn::CreatedAt,
                ])
                .values_panic([
                    id.into(),
                    create.event_id.into(),
                    create.level.into(),
                    create.message.into(),
                    create.created_at.into(),
                ]),
        )
        .await?;

        Ok(())
    }

    /// Query the logs for a specific event
    pub async fn query(
        db: &DbPool,
        event_id: Uuid,
        query: LogsQuery,
    ) -> DbResult<Vec<EventLogsModel>> {
        let mut select = Query::select();
        select
            .from(EventLogsTable)
            .columns([
                EventLogsColumn::Id,
                EventLogsColumn::EventId,
                EventLogsColumn::Level,
                EventLogsColumn::Message,
                EventLogsColumn::CreatedAt,
            ])
            .and_where(Expr::col(EventLogsColumn::EventId).eq(event_id))
            .and_where_option(
                query
                    .level
                    .map(|level| Expr::col(EventLogsColumn::Level).eq(level as i32)),
            )
            .and_where_option(
                query
                    .start_date
                    .map(|start_date| Expr::col(EventLogsColumn::CreatedAt).gt(start_date)),
            )
            .and_where_option(
                query
                    .end_date
                    .map(|end_date| Expr::col(EventLogsColumn::CreatedAt).lt(end_date)),
            )
            .order_by(EventLogsColumn::CreatedAt, Order::Desc);

        if let (Some(offset), Some(limit)) = (query.offset, query.limit) {
            select.offset(offset).limit(limit);
        }

        sql_query_all(db, &select).await
    }

    /// Deletes all logs that happened before the provided `start_time`.
    /// Used to clean out old logs
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(EventLogsTable)
                .and_where(Expr::col(EventLogsColumn::CreatedAt).lt(start_date)),
        )
        .await
    }

    /// Deletes a collection of specific logs by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(EventLogsTable)
                .and_where(Expr::col(EventLogsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Estimate the size of all log messages in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select().from(EventLogsTable).expr(Func::coalesce([
                // Get total length of all message text
                Func::sum(Func::char_length(Expr::col(EventLogsColumn::Message))).into(),
                // Fallback to zero
                Expr::value(0),
            ])),
        )
        .await
    }
}
