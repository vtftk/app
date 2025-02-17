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

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "event_executions")]
pub struct EventExecutionsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventExecutionsColumn {
    Id,
    EventId,
    Metadata,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventExecutionModel {
    pub id: Uuid,
    pub event_id: Uuid,
    #[sqlx(json)]
    pub metadata: EventExecutionMetadata,
    pub created_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub async fn create(db: &DbPool, create: CreateEventExecution) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let metadata_value = serde_json::to_value(&create.metadata)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(EventExecutionsTable)
                .columns([
                    EventExecutionsColumn::Id,
                    EventExecutionsColumn::EventId,
                    EventExecutionsColumn::Metadata,
                    EventExecutionsColumn::CreatedAt,
                ])
                .values_panic([
                    id.into(),
                    create.event_id.into(),
                    metadata_value.into(),
                    create.created_at.into(),
                ]),
        )
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
        sql_query_maybe_one(
            db,
            Query::select()
                .from(EventExecutionsTable)
                .columns([
                    EventExecutionsColumn::Id,
                    EventExecutionsColumn::EventId,
                    EventExecutionsColumn::Metadata,
                    EventExecutionsColumn::CreatedAt,
                ])
                .and_where(Expr::col(EventExecutionsColumn::EventId).eq(event_id))
                .offset(offset)
                .order_by(EventExecutionsColumn::CreatedAt, Order::Desc),
        )
        .await
    }

    /// Query the executions for a specific event
    pub async fn query(
        db: &DbPool,
        event_id: Uuid,
        query: ExecutionsQuery,
    ) -> DbResult<Vec<EventExecutionModel>> {
        let mut select = Query::select();
        select
            .from(EventExecutionsTable)
            .columns([
                EventExecutionsColumn::Id,
                EventExecutionsColumn::EventId,
                EventExecutionsColumn::Metadata,
                EventExecutionsColumn::CreatedAt,
            ])
            .and_where(Expr::col(EventExecutionsColumn::EventId).eq(event_id))
            .order_by(EventExecutionsColumn::CreatedAt, Order::Desc);

        if let Some(start_date) = query.start_date {
            select.and_where(Expr::col(EventExecutionsColumn::CreatedAt).gt(start_date));
        }

        if let Some(end_date) = query.end_date {
            select.and_where(Expr::col(EventExecutionsColumn::CreatedAt).lt(end_date));
        }

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
                .from_table(EventExecutionsTable)
                .and_where(Expr::col(EventExecutionsColumn::CreatedAt).lt(start_date)),
        )
        .await
    }

    /// Deletes a collection of specific executions by ID
    pub async fn delete_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(EventExecutionsTable)
                .and_where(Expr::col(EventExecutionsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Estimate the size of all execution metadata in the database
    pub async fn estimated_size(db: &DbPool) -> DbResult<u32> {
        sql_query_one_single(
            db,
            Query::select()
                .from(EventExecutionsTable)
                .expr(Func::coalesce([
                    // Get total length of all metadata text
                    Func::sum(Func::char_length(Expr::col(
                        EventExecutionsColumn::Metadata,
                    )))
                    .into(),
                    // Fallback to zero
                    Expr::value(0),
                ])),
        )
        .await
    }
}
