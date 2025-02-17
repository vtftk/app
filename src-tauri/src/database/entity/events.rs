use chrono::{DateTime, Utc};
use sea_query::{Alias, CaseStatement, Expr, Func, IdenStatic, Order, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::{
    database::{DbPool, DbResult},
    events::TwitchEventUser,
};

use super::shared::{
    ExecutionsQuery, LoggingLevelDb, LogsQuery, MinMax, MinimumRequireRole, UpdateOrdering,
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Whether the event is enabled
    pub enabled: bool,
    /// Name of the event handler
    pub name: String,
    /// Duplicate of the "trigger" column but just the string key to allow querying
    /// derived from "trigger"
    #[serde(skip)]
    pub trigger_type: EventTriggerType,
    /// Input that should trigger the event
    #[sqlx(json)]
    pub trigger: EventTrigger,
    /// Outcome the event should trigger
    #[sqlx(json)]
    pub outcome: EventOutcome,
    /// Cooldown between each trigger of the even
    #[sqlx(json)]
    pub cooldown: EventCooldown,
    /// Minimum required role to trigger the event
    pub require_role: MinimumRequireRole,
    /// Delay before executing the outcome
    pub outcome_delay: u32,
    /// Ordering
    pub order: u32,

    // Date time of creation
    pub created_at: DateTime<Utc>,
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
pub struct CreateEventExecution {
    pub event_id: Uuid,
    pub metadata: EventExecutionMetadata,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventCooldown {
    pub enabled: bool,
    pub duration: u32,
    pub per_user: bool,
}

impl Default for EventCooldown {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 0,
            per_user: false,
        }
    }
}

/// Copy of the [EventTrigger] enum but string variants to
/// support storing in the database as strings for querying
#[derive(
    Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, EnumString, Display,
)]
pub enum EventTriggerType {
    #[default]
    Redeem,
    Command,
    Follow,
    Subscription,
    GiftedSubscription,
    Bits,
    Raid,
    Timer,
    AdBreakBegin,
    ShoutoutReceive,
}

impl EventTriggerType {
    pub fn from_event_trigger(trigger: &EventTrigger) -> Self {
        match trigger {
            EventTrigger::Redeem { .. } => EventTriggerType::Redeem,
            EventTrigger::Command { .. } => EventTriggerType::Command,
            EventTrigger::Follow => EventTriggerType::Follow,
            EventTrigger::Subscription => EventTriggerType::Subscription,
            EventTrigger::GiftedSubscription => EventTriggerType::GiftedSubscription,
            EventTrigger::Bits { .. } => EventTriggerType::Bits,
            EventTrigger::Raid { .. } => EventTriggerType::Raid,
            EventTrigger::Timer { .. } => EventTriggerType::Timer,
            EventTrigger::AdBreakBegin => EventTriggerType::AdBreakBegin,
            EventTrigger::ShoutoutReceive { .. } => EventTriggerType::ShoutoutReceive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventTrigger {
    /// Redeem was triggered
    Redeem {
        /// ID of the reward required
        reward_id: String,
    },
    /// Command was sent
    Command {
        /// Command message required
        message: String,
    },
    /// User followed
    Follow,
    /// User subscribed
    Subscription,
    /// User gifted subscription
    GiftedSubscription,
    /// User gifts bits
    Bits {
        /// Minimum bits to trigger the event
        min_bits: u32,
    },
    /// Channel has been raided
    Raid {
        /// Minimum raiders required to trigger
        min_raiders: u32,
    },

    /// Run the event automatically on a fixed interval timer
    Timer {
        /// Interval in seconds to run
        interval: u64,

        /// Minimum chat messages that must have been received between each interval
        /// for the timer to trigger to prevent spamming when nobody is chatting
        #[serde(default)]
        min_chat_messages: u32,
    },

    /// Ad break started
    AdBreakBegin,

    /// Shoutout received
    ShoutoutReceive {
        /// Minimum viewers required
        min_viewers: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ThrowableAmountData {
    /// Throw items (All at once)
    Throw {
        /// Amount of items to throw
        amount: i64,

        /// Override to derive amount of items to throw
        #[serde(default)]
        use_input_amount: bool,
        /// Additional configuration for when use_input_amount is true
        #[serde(default)]
        input_amount_config: InputAmountConfig,
    },

    /// Throw a throwable barrage
    Barrage {
        /// Amount to throw for each throw
        amount_per_throw: u32,
        /// Time between each thrown item (Milliseconds)
        frequency: u32,
        /// Total amount of items to throw
        amount: i64,

        /// Override to derive amount of items to throw
        #[serde(default)]
        use_input_amount: bool,
        /// Additional configuration for when use_input_amount is true
        #[serde(default)]
        input_amount_config: InputAmountConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputAmountConfig {
    /// Multiplier to apply against the input amount
    pub multiplier: f64,
    /// Allowed range for the input
    pub range: MinMax<i64>,
}

impl Default for InputAmountConfig {
    fn default() -> Self {
        Self {
            multiplier: 1.,
            range: MinMax { min: 1, max: 100 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeBits {
    /// Throwable to throw for 1 bit (Override, defaults to builtin)
    pub _1: Option<Uuid>,
    /// Throwable to throw for 100 bits (Override, defaults to builtin)
    pub _100: Option<Uuid>,
    /// Throwable to throw for 1000 bits (Override, defaults to builtin)
    pub _1000: Option<Uuid>,
    /// Throwable to throw for 5000 bits (Override, defaults to builtin)
    pub _5000: Option<Uuid>,
    /// Throwable to throw for 10000 bits (Override, defaults to builtin)
    pub _10000: Option<Uuid>,
    /// How many bits to throw
    pub amount: ThrowableAmountData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeThrowable {
    /// IDs of the throwables to throw
    pub throwable_ids: Vec<Uuid>,
    /// Throwable data
    #[serde(alias = "data")]
    pub amount: ThrowableAmountData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeTriggerHotkey {
    pub hotkey_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomePlaySound {
    pub sound_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeSendChat {
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeScript {
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeChannelEmotes {
    /// How many emotes to throw
    pub amount: ThrowableAmountData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventOutcome {
    /// Throw bits (Only compatible with bits trigger)
    ThrowBits(EventOutcomeBits),
    /// Throw something
    Throwable(EventOutcomeThrowable),
    /// Trigger a VTube studio hotkey
    TriggerHotkey(EventOutcomeTriggerHotkey),
    /// Trigger a sound
    PlaySound(EventOutcomePlaySound),
    /// Send a chat message
    SendChatMessage(EventOutcomeSendChat),
    /// Execute a script
    Script(EventOutcomeScript),
    /// Throw the emotes of a specific channel
    ChannelEmotes(EventOutcomeChannelEmotes),
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub enabled: bool,
    pub name: String,
    pub trigger: EventTrigger,
    pub outcome: EventOutcome,
    pub cooldown: EventCooldown,
    pub require_role: MinimumRequireRole,
    pub outcome_delay: u32,
}

#[derive(Default, Deserialize)]
pub struct UpdateEvent {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub trigger: Option<EventTrigger>,
    pub outcome: Option<EventOutcome>,
    pub cooldown: Option<EventCooldown>,
    pub require_role: Option<MinimumRequireRole>,
    pub outcome_delay: Option<u32>,
    pub order: Option<u32>,
}

#[derive(Debug)]
pub struct CreateEventLog {
    pub event_id: Uuid,
    pub level: LoggingLevelDb,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl EventModel {
    fn columns() -> [EventsColumn; 11] {
        [
            EventsColumn::Id,
            EventsColumn::Enabled,
            EventsColumn::Name,
            EventsColumn::TriggerType,
            EventsColumn::Trigger,
            EventsColumn::Outcome,
            EventsColumn::Cooldown,
            EventsColumn::RequireRole,
            EventsColumn::OutcomeDelay,
            EventsColumn::Order,
            EventsColumn::CreatedAt,
        ]
    }

    /// Create a new event
    pub async fn create(db: &DbPool, create: CreateEvent) -> anyhow::Result<EventModel> {
        let id = Uuid::new_v4();
        let model = EventModel {
            id,
            enabled: create.enabled,
            name: create.name,
            trigger_type: EventTriggerType::from_event_trigger(&create.trigger),
            trigger: create.trigger,
            outcome: create.outcome,
            cooldown: create.cooldown,
            require_role: create.require_role,
            outcome_delay: create.outcome_delay,
            order: 0,
            created_at: Utc::now(),
        };

        let trigger_value = serde_json::to_value(&model.trigger)?;
        let outcome_value = serde_json::to_value(&model.outcome)?;

        let (sql, values) = Query::insert()
            .into_table(EventsTable)
            .columns(EventModel::columns())
            .values_panic([
                model.id.into(),
                model.enabled.into(),
                model.name.clone().into(),
                model.trigger_type.to_string().into(),
                trigger_value.into(),
                outcome_value.into(),
                model.require_role.to_string().into(),
                model.outcome_delay.into(),
                model.order.into(),
                model.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(model)
    }

    /// Find a specific event by ID
    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<EventModel>> {
        let (sql, values) = Query::select()
            .columns(EventModel::columns())
            .from(EventsTable)
            .and_where(Expr::col(EventsColumn::Id).eq(id))
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(result)
    }

    /// Find the most recent execution of this event
    pub async fn last_execution(
        &self,
        db: &DbPool,
        offset: u64,
    ) -> DbResult<Option<EventExecutionModel>> {
        let (sql, values) = Query::select()
            .from(EventExecutionsTable)
            .columns([
                EventExecutionsColumn::Id,
                EventExecutionsColumn::EventId,
                EventExecutionsColumn::Metadata,
                EventExecutionsColumn::CreatedAt,
            ])
            .and_where(Expr::col(EventExecutionsColumn::EventId).eq(self.id))
            .offset(offset)
            .order_by(EventExecutionsColumn::CreatedAt, Order::Desc)
            .build_sqlx(SqliteQueryBuilder);

        let value: Option<EventExecutionModel> =
            sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(value)
    }

    /// Find a specific event by a specific trigger type
    ///
    /// Filters to only events marked as enabled
    pub async fn get_by_trigger_type(
        db: &DbPool,
        trigger_type: EventTriggerType,
    ) -> DbResult<Vec<Self>> {
        let (sql, values) = Query::select()
            .columns(EventModel::columns())
            .from(EventsTable)
            .and_where(Expr::col(EventsColumn::TriggerType).eq(trigger_type.to_string()))
            .and_where(Expr::col(EventsColumn::Enabled).eq(true))
            .order_by_columns([
                (EventsColumn::Order, Order::Asc),
                (EventsColumn::CreatedAt, Order::Desc),
            ])
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(result)
    }

    /// Find all events
    pub async fn all(db: &DbPool) -> DbResult<Vec<Self>> {
        let (sql, values) = Query::select()
            .columns(EventModel::columns())
            .from(EventsTable)
            .order_by_columns([
                (EventsColumn::Order, Order::Asc),
                (EventsColumn::CreatedAt, Order::Desc),
            ])
            .build_sqlx(SqliteQueryBuilder);
        let result = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(result)
    }

    /// Update the current event
    pub async fn update(&mut self, db: &DbPool, data: UpdateEvent) -> anyhow::Result<()> {
        let mut update = Query::update();
        update.table(EventsTable);

        if let Some(enabled) = data.enabled {
            self.enabled = enabled;
            update.value(EventsColumn::Enabled, Expr::value(enabled));
        }

        if let Some(name) = data.name {
            self.name = name.clone();
            update.value(EventsColumn::Name, Expr::value(name));
        }

        if let Some(trigger) = data.trigger {
            self.trigger_type = EventTriggerType::from_event_trigger(&trigger);
            self.trigger = trigger;

            let trigger_value = serde_json::to_value(&self.trigger)?;
            update.value(EventsColumn::Trigger, Expr::value(trigger_value));
        }

        if let Some(outcome) = data.outcome {
            self.outcome = outcome;

            let outcome_value = serde_json::to_value(&self.outcome)?;
            update.value(EventsColumn::Outcome, Expr::value(outcome_value));
        }

        if let Some(cooldown) = data.cooldown {
            self.cooldown = cooldown;

            let cooldown_value = serde_json::to_value(&self.cooldown)?;
            update.value(EventsColumn::Cooldown, Expr::value(cooldown_value));
        }

        if let Some(require_role) = data.require_role {
            self.require_role = require_role;
            update.value(
                EventsColumn::RequireRole,
                Expr::value(require_role.to_string()),
            );
        }

        if let Some(outcome_delay) = data.outcome_delay {
            self.outcome_delay = outcome_delay;
            update.value(EventsColumn::OutcomeDelay, Expr::value(outcome_delay));
        }

        if let Some(order) = data.order {
            self.order = order;
            update.value(EventsColumn::Order, Expr::value(order));
        }

        let (sql, values) = update.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(())
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let mut case = CaseStatement::new()
                // Use the current column value when not specified
                .finally(Expr::col(EventsColumn::Order));

            // Add case for all updated values
            for order in order_chunk {
                case = case.case(
                    Expr::col(EventsColumn::Id).eq(order.id),
                    Expr::value(order.order),
                );
            }

            let (sql, values) = Query::update()
                .table(EventsTable)
                .value(EventsColumn::Order, case)
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(db).await?;
        }

        Ok(())
    }

    pub async fn get_executions(
        &self,
        db: &DbPool,
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
            .and_where(Expr::col(EventExecutionsColumn::EventId).eq(self.id))
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

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        let results = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results)
    }

    pub async fn get_logs(&self, db: &DbPool, query: LogsQuery) -> DbResult<Vec<EventLogsModel>> {
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
            .and_where(Expr::col(EventLogsColumn::EventId).eq(self.id))
            .order_by(EventLogsColumn::CreatedAt, Order::Desc);

        if let Some(level) = query.level {
            select.and_where(Expr::col(EventLogsColumn::Level).eq(level as i32));
        }

        if let Some(start_date) = query.start_date {
            select.and_where(Expr::col(EventLogsColumn::CreatedAt).gt(start_date));
        }

        if let Some(end_date) = query.end_date {
            select.and_where(Expr::col(EventLogsColumn::CreatedAt).lt(end_date));
        }

        if let Some(offset) = query.offset {
            select.offset(offset);
        }

        if let Some(limit) = query.limit {
            select.limit(limit);
        }

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        let results = sqlx::query_as_with(&sql, values).fetch_all(db).await?;
        Ok(results)
    }

    /// Create a new script
    pub async fn create_log(db: &DbPool, create: CreateEventLog) -> DbResult<()> {
        let id = Uuid::new_v4();

        let (sql, values) = Query::insert()
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
                (create.level as i32).into(),
                create.message.to_string().into(),
                create.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(())
    }

    /// Create a new script
    pub async fn create_execution(
        db: &DbPool,
        create: CreateEventExecution,
    ) -> anyhow::Result<EventExecutionModel> {
        let id = Uuid::new_v4();
        let model = EventExecutionModel {
            id,
            event_id: create.event_id,
            metadata: create.metadata,
            created_at: create.created_at,
        };

        let metadata_value = serde_json::to_value(&model.metadata)?;

        let (sql, values) = Query::insert()
            .into_table(EventExecutionsTable)
            .columns([
                EventExecutionsColumn::Id,
                EventExecutionsColumn::EventId,
                EventExecutionsColumn::Metadata,
                EventExecutionsColumn::CreatedAt,
            ])
            .values_panic([
                model.id.into(),
                model.event_id.into(),
                metadata_value.into(),
                model.created_at.into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;

        Ok(model)
    }

    pub async fn delete_executions_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(EventExecutionsTable)
            .and_where(Expr::col(EventExecutionsColumn::CreatedAt).lt(start_date))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn delete_many_executions(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(EventExecutionsTable)
            .and_where(Expr::col(EventExecutionsColumn::Id).is_in(ids.iter().copied()))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn delete_many_logs(db: &DbPool, ids: &[Uuid]) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(EventLogsTable)
            .and_where(Expr::col(EventLogsColumn::Id).is_in(ids.iter().copied()))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn delete_logs_before(db: &DbPool, start_date: DateTime<Utc>) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(EventLogsTable)
            .and_where(Expr::col(EventLogsColumn::CreatedAt).lt(start_date))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }

    pub async fn get_logs_estimate_size(db: &DbPool) -> DbResult<u32> {
        #[derive(Default, FromRow)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let (sql, values) = Query::select()
            .from(EventLogsTable)
            .expr_as(
                Func::sum(Func::char_length(Expr::col(EventLogsColumn::Message))),
                Alias::new("total_message_length"),
            )
            .build_sqlx(SqliteQueryBuilder);

        let result: PartialModel = sqlx::query_as_with(&sql, values).fetch_one(db).await?;
        Ok(result.total_message_length.unwrap_or_default())
    }

    pub async fn get_executions_estimate_size(db: &DbPool) -> DbResult<u32> {
        #[derive(Default, FromRow)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let (sql, values) = Query::select()
            .from(EventExecutionsTable)
            .expr_as(
                Func::sum(Func::char_length(Expr::col(
                    EventExecutionsColumn::Metadata,
                ))),
                Alias::new("total_message_length"),
            )
            .build_sqlx(SqliteQueryBuilder);

        let result: PartialModel = sqlx::query_as_with(&sql, values).fetch_one(db).await?;
        Ok(result.total_message_length.unwrap_or_default())
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(EventsTable)
            .and_where(Expr::col(EventsColumn::Id).eq(self.id))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "events")]
pub struct EventsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventsColumn {
    Id,
    Enabled,
    Name,
    TriggerType,
    Trigger,
    Outcome,
    Cooldown,
    RequireRole,
    OutcomeDelay,
    Order,
    CreatedAt,
}

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
