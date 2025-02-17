use chrono::{DateTime, Utc};
use sea_query::{Expr, Func, IdenStatic, Order, Query};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    database::{
        helpers::{sql_exec, sql_query_all, sql_query_maybe_one, sql_query_one_single},
        DbPool, DbResult,
    },
    events::TwitchEventUser,
};

use super::shared::ExecutionsQuery;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommandExecutionModel {
    /// Unique ID for the event
    pub id: Uuid,
    pub command_id: Uuid,
    #[sqlx(json)]
    pub metadata: CommandExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionMetadata {
    /// User who triggered the event
    pub user: Option<TwitchEventUser>,

    /// Catchall for any other metadata
    #[serde(flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub data: Vec<(String, serde_json::Value)>,
}

#[derive(Debug)]
pub struct CreateCommandExecution {
    pub command_id: Uuid,
    pub metadata: CommandExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "command_executions")]
pub struct CommandExecutionsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum CommandExecutionsColumn {
    Id,
    CommandId,
    Metadata,
    CreatedAt,
}

impl CommandExecutionModel {
    /// Create an execution for a specific command
    pub async fn create(db: &DbPool, create: CreateCommandExecution) -> anyhow::Result<()> {
        let id = Uuid::new_v4();

        let metadata_value = serde_json::to_value(&create.metadata)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(CommandExecutionsTable)
                .columns([
                    CommandExecutionsColumn::Id,
                    CommandExecutionsColumn::CommandId,
                    CommandExecutionsColumn::Metadata,
                    CommandExecutionsColumn::CreatedAt,
                ])
                .values_panic([
                    id.into(),
                    create.command_id.into(),
                    metadata_value.into(),
                    create.created_at.into(),
                ]),
        )
        .await?;

        Ok(())
    }

    /// Find the most recent execution of a command, with an offset
    /// to get the nth recent execution
    pub async fn last(
        db: &DbPool,
        command_id: Uuid,
        offset: u64,
    ) -> DbResult<Option<CommandExecutionModel>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .from(CommandExecutionsTable)
                .columns([
                    CommandExecutionsColumn::Id,
                    CommandExecutionsColumn::CommandId,
                    CommandExecutionsColumn::Metadata,
                    CommandExecutionsColumn::CreatedAt,
                ])
                .and_where(Expr::col(CommandExecutionsColumn::CommandId).eq(command_id))
                .offset(offset)
                .order_by(CommandExecutionsColumn::CreatedAt, Order::Desc),
        )
        .await
    }

    /// Query the executions for a specific command
    pub async fn query(
        db: &DbPool,
        command_id: Uuid,
        query: ExecutionsQuery,
    ) -> DbResult<Vec<CommandExecutionModel>> {
        let mut select = Query::select();
        select
            .from(CommandExecutionsTable)
            .columns([
                CommandExecutionsColumn::Id,
                CommandExecutionsColumn::CommandId,
                CommandExecutionsColumn::Metadata,
                CommandExecutionsColumn::CreatedAt,
            ])
            .and_where(Expr::col(CommandExecutionsColumn::CommandId).eq(command_id))
            .and_where_option(
                query
                    .start_date
                    .map(|start_date| Expr::col(CommandExecutionsColumn::CreatedAt).gt(start_date)),
            )
            .and_where_option(
                query
                    .end_date
                    .map(|end_date| Expr::col(CommandExecutionsColumn::CreatedAt).lt(end_date)),
            )
            .order_by(CommandExecutionsColumn::CreatedAt, Order::Desc);

        if let Some(offset) = query.offset {
            select.offset(offset);
        }

        if let Some(limit) = query.limit {
            select.limit(limit);
        }

        sql_query_all(db, &select).await
    }

    /// Deletes all executions that happened before the provided `start_time`.
    /// Used to clean out old executions
    pub async fn delete_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandExecutionsTable)
                .and_where(Expr::col(CommandExecutionsColumn::CreatedAt).lt(start_date)),
        )
        .await
    }

    /// Deletes a collection of specific executions by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(CommandExecutionsTable)
                .and_where(Expr::col(CommandExecutionsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Estimate the size of all execution metadata in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select()
                .from(CommandExecutionsTable)
                .expr(Func::coalesce([
                    // Get total length of all metadata text
                    Func::sum(Func::char_length(Expr::col(
                        CommandExecutionsColumn::Metadata,
                    )))
                    .into(),
                    // Fallback to zero
                    Expr::value(0),
                ])),
        )
        .await
    }
}
