use chrono::{DateTime, Utc};
use sea_query::{CaseStatement, Expr, IdenStatic, Order, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_maybe_one},
    DbPool, DbResult,
};

use super::shared::{MinMax, MinimumRequireRole, UpdateOrdering};

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "events")]
pub struct EventsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum EventsColumn {
    Id,
    Enabled,
    Name,
    TriggerType,
    Config,
    Order,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Whether the event is enabled
    pub enabled: bool,
    /// Name of the event handler
    pub name: String,
    #[sqlx(json)]
    pub config: EventConfig,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Input that should trigger the event
    pub trigger: EventTrigger,
    /// Outcome the event should trigger
    pub outcome: EventOutcome,
    /// Cooldown between each trigger of the even
    pub cooldown: EventCooldown,
    /// Minimum required role to trigger the event
    pub require_role: MinimumRequireRole,
    /// Delay before executing the outcome
    pub outcome_delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type, EnumString, Display)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomeThrowable {
    /// IDs of the throwables to throw
    pub throwable_ids: Vec<Uuid>,
    /// Throwable data
    #[serde(alias = "data")]
    pub amount: ThrowableAmountData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomeTriggerHotkey {
    pub hotkey_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomePlaySound {
    pub sound_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomeSendChat {
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomeScript {
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcomeChannelEmotes {
    /// How many emotes to throw
    pub amount: ThrowableAmountData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub config: EventConfig,
}

#[derive(Default, Deserialize)]
pub struct UpdateEvent {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub config: Option<EventConfig>,
    pub order: Option<u32>,
}

impl EventModel {
    fn columns() -> [EventsColumn; 6] {
        [
            EventsColumn::Id,
            EventsColumn::Enabled,
            EventsColumn::Name,
            EventsColumn::Config,
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
            config: create.config,
            order: 0,
            created_at: Utc::now(),
        };

        let config_value = serde_json::to_value(&model.config)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(EventsTable)
                .columns([
                    EventsColumn::Id,
                    EventsColumn::Enabled,
                    EventsColumn::Name,
                    EventsColumn::Config,
                    EventsColumn::Order,
                    EventsColumn::CreatedAt,
                ])
                .values_panic([
                    model.id.into(),
                    model.enabled.into(),
                    model.name.clone().into(),
                    config_value.into(),
                    model.order.into(),
                    model.created_at.into(),
                ]),
        )
        .await?;

        Ok(model)
    }

    /// Find a specific event by ID
    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<EventModel>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .columns(EventModel::columns())
                .from(EventsTable)
                .and_where(Expr::col(EventsColumn::Id).eq(id)),
        )
        .await
    }

    /// Find a specific event by a specific trigger type
    ///
    /// Filters to only events marked as enabled
    pub async fn get_by_trigger_type(
        db: &DbPool,
        trigger_type: EventTriggerType,
    ) -> DbResult<Vec<EventModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns(EventModel::columns())
                .from(EventsTable)
                .and_where(Expr::col(EventsColumn::TriggerType).eq(trigger_type.to_string()))
                .and_where(Expr::col(EventsColumn::Enabled).eq(true))
                .order_by_columns([
                    (EventsColumn::Order, Order::Asc),
                    (EventsColumn::CreatedAt, Order::Desc),
                ]),
        )
        .await
    }

    /// Find all events
    pub async fn all(db: &DbPool) -> DbResult<Vec<EventModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns(EventModel::columns())
                .from(EventsTable)
                .order_by_columns([
                    (EventsColumn::Order, Order::Asc),
                    (EventsColumn::CreatedAt, Order::Desc),
                ]),
        )
        .await
    }

    /// Update the current event
    pub async fn update(&mut self, db: &DbPool, data: UpdateEvent) -> anyhow::Result<()> {
        let mut update = Query::update();
        update
            .table(EventsTable)
            .and_where(Expr::col(EventsColumn::Id).eq(self.id));

        if let Some(enabled) = data.enabled {
            self.enabled = enabled;
            update.value(EventsColumn::Enabled, Expr::value(enabled));
        }

        if let Some(name) = data.name {
            self.name = name.clone();
            update.value(EventsColumn::Name, Expr::value(name));
        }

        if let Some(config) = data.config {
            self.config = config;

            let config_value = serde_json::to_value(&self.config)?;
            update.value(EventsColumn::Config, Expr::value(config_value));
        }

        if let Some(order) = data.order {
            self.order = order;
            update.value(EventsColumn::Order, Expr::value(order));
        }

        sql_exec(db, &update).await?;

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

            sql_exec(
                db,
                Query::update()
                    .table(EventsTable)
                    .value(EventsColumn::Order, case),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(EventsTable)
                .and_where(Expr::col(EventsColumn::Id).eq(self.id)),
        )
        .await
    }
}
