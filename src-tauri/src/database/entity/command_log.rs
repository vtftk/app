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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
pub struct CreateCommandLog {
    pub command_id: Uuid,
    pub level: LoggingLevelDb,
    pub message: String,
    pub created_at: DateTime<Utc>,
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

impl CommandLogsModel {
    /// Create a new log for a specific command
    pub async fn create(db: &DbPool, create: CreateCommandLog) -> DbResult<()> {
        let id = Uuid::new_v4();

        sql_exec(
            db,
            Query::insert()
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
                ]),
        )
        .await?;

        Ok(())
    }

    /// Query the logs for a specific command
    pub async fn query(
        db: &DbPool,
        command_id: Uuid,
        query: LogsQuery,
    ) -> DbResult<Vec<CommandLogsModel>> {
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
            .and_where(Expr::col(CommandLogsColumn::CommandId).eq(command_id))
            .and_where_option(
                query
                    .level
                    .map(|level| Expr::col(CommandLogsColumn::Level).eq(level as i32)),
            )
            .and_where_option(
                query
                    .start_date
                    .map(|start_date| Expr::col(CommandLogsColumn::CreatedAt).gt(start_date)),
            )
            .and_where_option(
                query
                    .end_date
                    .map(|end_date| Expr::col(CommandLogsColumn::CreatedAt).lt(end_date)),
            )
            .order_by(CommandLogsColumn::CreatedAt, Order::Desc);

        if let Some(offset) = query.offset {
            select.offset(offset);
        }

        if let Some(limit) = query.limit {
            select.limit(limit);
        }

        sql_query_all(db, &select).await
    }

    /// Deletes all logs that happened before the provided `start_time`.
    /// Used to clean out old logs
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandLogsTable)
                .and_where(Expr::col(CommandLogsColumn::CreatedAt).lt(start_date)),
        )
        .await
    }

    /// Deletes a collection of specific logs by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandLogsTable)
                .and_where(Expr::col(CommandLogsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Estimate the size of all log messages in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select().from(CommandLogsTable).expr(Func::coalesce([
                // Get total length of all message text
                Func::sum(Func::char_length(Expr::col(CommandLogsColumn::Message))).into(),
                // Fallback to zero
                Expr::value(0),
            ])),
        )
        .await
    }
}
