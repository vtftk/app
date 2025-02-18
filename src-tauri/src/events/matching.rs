use std::str::FromStr;

use anyhow::{bail, Context};
use chrono::Utc;
use log::{debug, error};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::join;
use twitch_api::{
    eventsub::channel::chat::Fragment,
    types::{MsgId, SubscriptionTier},
};
use uuid::Uuid;

use crate::{
    database::{
        entity::{
            chat_history::{ChatHistoryModel, CreateChatHistory},
            commands::CommandModel,
            event_execution::EventExecutionModel,
            events::{EventModel, EventTrigger, EventTriggerType},
        },
        DbPool,
    },
    events::{
        TwitchEventAdBreakBegin, TwitchEventChatMsg, TwitchEventCheerBits, TwitchEventFollow,
        TwitchEventGiftSub, TwitchEventRaid, TwitchEventReSub, TwitchEventRedeem,
        TwitchEventShoutoutReceive, TwitchEventSub, TwitchEventUser,
    },
    twitch::manager::Twitch,
};

use super::TimerCompleted;

/// Data for matched events to trigger
#[derive(Default)]
pub struct EventMatchingData {
    /// List of events to attempt to trigger
    pub events: Vec<EventModel>,

    /// List of commands to trigger
    pub commands: Vec<CommandWithContext>,

    /// Additional data attached to the event
    pub event_data: EventData,
}

/// Command to trigger with some additional context
pub struct CommandWithContext {
    pub command: CommandModel,

    /// Message with the command/alias removed
    pub message: String,

    /// Args with the first argument command/alias removed
    pub args: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// User who triggered the event
    pub user: Option<TwitchEventUser>,

    /// Additional input data
    #[serde(flatten)]
    pub input_data: EventInputData,
}

/// Additional event-specific input data
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventInputData {
    /// Redeems specific data
    #[serde(rename_all = "camelCase")]
    Redeem {
        /// Unique ID of the redemption itself
        redemption_id: String,
        /// ID of the redeemed award
        reward_id: String,
        /// Name of the reward
        reward_name: String,
        /// Cost of the reward
        cost: i64,
        /// User provided message (For redeems that let you provide a message)
        user_input: String,
    },

    /// Bits specific data
    #[serde(rename_all = "camelCase")]
    Bits {
        /// Number of bits given
        bits: i64,
        /// Whether the bits were given anonymously
        anonymous: bool,
        /// User message provided alongside the bits
        message: String,
    },

    /// Subscription specific data
    #[serde(rename_all = "camelCase")]
    Subscription {
        /// Tier subscribed at
        tier: SubscriptionTier,
        /// Whether the subscription was gifted
        is_gift: bool,
    },

    /// Gifted Subscription specific data
    #[serde(rename_all = "camelCase")]
    GiftedSubscription {
        /// Gifted subscription tier
        tier: SubscriptionTier,
        /// Total gifts user has given (If not anonymous)
        cumulative_total: Option<i64>,
        /// Whether the gifted were given anonymously
        anonymous: bool,
        /// Total subs gifted
        total: i64,
    },

    /// Re-Subscription specific data
    #[serde(rename_all = "camelCase")]
    ReSubscription {
        /// The total number of months the user has been subscribed to the channel.
        cumulative_months: i64,
        /// The month duration of the subscription.
        duration_months: i64,
        /// User message attached to the resubscription
        message: String,
        /// The number of consecutive months the user’s current subscription has been active.
        /// This value is null if the user has opted out of sharing this information.
        streak_months: Option<i64>,
        /// Gifted subscription tier
        tier: SubscriptionTier,
    },

    /// Chat message specific data
    #[serde(rename_all = "camelCase")]
    Chat {
        /// ID of the chat message
        message_id: MsgId,

        /// The chat message itself
        message: String,

        /// Chat message fragments
        fragments: Vec<Fragment>,

        /// Optional amount of bits cheered (If user cheered bits)
        cheer: Option<usize>,
    },

    /// Raid specific data
    #[serde(rename_all = "camelCase")]
    Raid {
        /// The number of viewers in the raid.
        viewers: i64,
    },

    /// Ad break specific data
    #[serde(rename_all = "camelCase")]
    AdBreakBegin {
        /// Duration of the ad break in seconds
        duration_seconds: i32,
    },

    /// Shoutout specific data
    #[serde(rename_all = "camelCase")]
    ShoutoutReceive {
        /// Number of viewers in the shoutout
        viewer_count: i64,
    },

    /// No additional input data
    #[default]
    #[serde(deserialize_with = "deserialize_ignore_any")]
    None,
}

pub fn deserialize_ignore_any<'de, D: Deserializer<'de>, T: Default>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde::de::IgnoredAny::deserialize(deserializer).map(|_| T::default())
}

pub async fn match_redeem_event(
    db: &DbPool,
    event: TwitchEventRedeem,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Redeem).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    // Get requested reward ID
    let event_reward_id = event.reward.id.to_string();

    // Filter events for the matching reward ID
    let events = events
        .into_iter()
        .filter(|event| {
            matches!(
                &event.config.trigger,
                EventTrigger::Redeem { reward_id } if event_reward_id.eq(reward_id)
            )
        })
        .collect();

    let event_data = EventData {
        input_data: EventInputData::Redeem {
            redemption_id: event.id.to_string(),
            reward_id: event_reward_id,
            reward_name: event.reward.title.clone(),
            cost: event.reward.cost,
            user_input: event.user_input,
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_cheer_bits_event(
    db: &DbPool,
    event: TwitchEventCheerBits,
) -> anyhow::Result<EventMatchingData> {
    let bits = event.bits;

    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Bits).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    // Filter events for the matching reward ID
    let events = events
        .into_iter()
        .filter(|event| {
            matches!(&event.config.trigger, EventTrigger::Bits { min_bits } if bits >= *min_bits as i64)
        })
        .collect();

    // Create user (Bits can be anonymous)
    let user = match (event.user_id, event.user_name, event.user_display_name) {
        (Some(user_id), Some(user_name), Some(user_display_name)) => Some(TwitchEventUser {
            id: user_id,
            name: user_name,
            display_name: user_display_name,
        }),
        _ => None,
    };

    let event_data = EventData {
        input_data: EventInputData::Bits {
            bits: event.bits,
            anonymous: event.anonymous,
            message: event.message,
        },
        user,
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_follow_event(
    db: &DbPool,
    event: TwitchEventFollow,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Follow).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let event_data = EventData {
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
        ..Default::default()
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_subscription_event(
    db: &DbPool,
    event: TwitchEventSub,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Subscription).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let event_data = EventData {
        input_data: EventInputData::Subscription {
            tier: event.tier,
            is_gift: event.is_gift,
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_gifted_subscription_event(
    db: &DbPool,
    event: TwitchEventGiftSub,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::GiftedSubscription).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    // Create user (Bits can be anonymous)
    let user = match (event.user_id, event.user_name, event.user_display_name) {
        (Some(user_id), Some(user_name), Some(user_display_name)) => Some(TwitchEventUser {
            id: user_id,
            name: user_name,
            display_name: user_display_name,
        }),
        _ => None,
    };

    let event_data = EventData {
        input_data: EventInputData::GiftedSubscription {
            tier: event.tier,
            cumulative_total: event.cumulative_total,
            anonymous: event.anonymous,
            total: event.total,
        },
        user,
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_re_subscription_event(
    db: &DbPool,
    event: TwitchEventReSub,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Subscription).await;
    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let event_data = EventData {
        input_data: EventInputData::ReSubscription {
            cumulative_months: event.cumulative_months,
            duration_months: event.duration_months,
            message: event.message.text,
            streak_months: event.streak_months,
            tier: event.tier,
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

async fn store_chat_event(db: &DbPool, event: &TwitchEventChatMsg) -> anyhow::Result<()> {
    let id = Uuid::from_str(event.message_id.as_str()).context("invalid message ID")?;

    ChatHistoryModel::create(
        db,
        CreateChatHistory {
            id,
            user_id: event.user_id.to_string(),
            message: event.message.text.clone(),
            cheer: event.cheer.as_ref().map(|value| value.bits as u32),
            created_at: Utc::now(),
        },
    )
    .await?;

    Ok(())
}

pub async fn match_chat_event(
    db: &DbPool,
    event: TwitchEventChatMsg,
) -> anyhow::Result<EventMatchingData> {
    if let Err(err) = store_chat_event(db, &event).await {
        error!("failed to log chat message history: {err}");
    }

    let message = event.message.text.clone();
    let mut args: Vec<String> = message
        .split_whitespace()
        .map(|value| value.to_string())
        .collect();

    let first_arg = if args.is_empty() {
        None
    } else {
        Some(args.remove(0))
    };

    let (events, commands) = if let Some(first_arg) = first_arg {
        // Get the command argument from the first argument
        let command_arg = first_arg.trim().to_lowercase();

        let (events, commands) = join!(
            // Load all command event triggers
            EventModel::get_by_trigger_type(db, EventTriggerType::Command),
            // Load all commands
            CommandModel::get_by_command(db, &command_arg),
        );

        let events = match events {
            Ok(value) => value,
            Err(err) => {
                error!("failed to load events: {:?}", err);
                Default::default()
            }
        };

        let commands = match commands {
            Ok(value) => value,
            Err(err) => {
                error!("failed to load commands: {:?}", err);
                Default::default()
            }
        };

        // Filter events for matching command messages
        let events = events
            .into_iter()
            .filter(|event| matches!(&event.config.trigger, EventTrigger::Command { message } if message.trim().to_lowercase().eq(&command_arg)))
            .collect();

        // Provide additional context to commands
        let commands = commands
            .into_iter()
            .map(|command| {
                // Strip prefix and trim any leading space
                let message = message
                    .strip_prefix(&first_arg)
                    .unwrap_or(&message)
                    .trim_start()
                    .to_string();

                CommandWithContext {
                    command,
                    message,
                    args: args.clone(),
                }
            })
            .collect();

        (events, commands)
    } else {
        (Default::default(), Default::default())
    };

    let event_data = EventData {
        input_data: EventInputData::Chat {
            message_id: event.message_id,
            message: event.message.text,
            fragments: event.message.fragments,
            cheer: event.cheer.map(|cheer| cheer.bits),
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands,
        event_data,
    })
}

pub async fn match_raid_event(
    db: &DbPool,
    event: TwitchEventRaid,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::Raid).await;

    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let raiders = event.viewers;

    // Filter events for the matching viewer minimum amount
    let events = events
       .into_iter()
       .filter(|event| {
           matches!(&event.config.trigger, EventTrigger::Raid { min_raiders } if raiders >= *min_raiders as i64)
       })
       .collect();

    let event_data = EventData {
        input_data: EventInputData::Raid {
            viewers: event.viewers,
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_ad_break_event(
    db: &DbPool,
    event: TwitchEventAdBreakBegin,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::AdBreakBegin).await;

    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let event_data = EventData {
        input_data: EventInputData::AdBreakBegin {
            duration_seconds: event.duration_seconds,
        },
        user: None,
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

pub async fn match_timer_complete_event(
    db: &DbPool,
    twitch: &Twitch,
    event: TimerCompleted,
) -> anyhow::Result<EventMatchingData> {
    let event = EventModel::get_by_id(db, event.event_id)
        .await
        .context("failed to get event to trigger")?
        .context("requested timer event does not exist")?;

    // Get minimum amount of chat messages required
    let min_chat_messages = match &event.config.trigger {
        EventTrigger::Timer {
            min_chat_messages, ..
        } => *min_chat_messages,
        _ => {
            bail!("attempted to execute timer event that was not a timer event");
        }
    };

    let user_id = twitch.get_user_id().await;

    // Ensure minimum chat messages has been reached
    if min_chat_messages > 0 {
        let last_execution = EventExecutionModel::last(db, event.id, 0)
            .await
            .context("failed to get last execution")?;

        if let Some(last_execution) = last_execution {
            let message_count =
                ChatHistoryModel::count_since(db, last_execution.created_at, user_id).await?;

            if message_count < min_chat_messages {
                debug!("skipping timer execution, not enough chat messages since last execution");
                return Ok(Default::default());
            }
        }
    }

    Ok(EventMatchingData {
        events: vec![event],
        commands: Default::default(),
        event_data: Default::default(),
    })
}

pub async fn match_shoutout_receive_event(
    db: &DbPool,
    event: TwitchEventShoutoutReceive,
) -> anyhow::Result<EventMatchingData> {
    let events = EventModel::get_by_trigger_type(db, EventTriggerType::ShoutoutReceive).await;

    let events = match events {
        Ok(value) => value,
        Err(err) => {
            error!("failed to load events: {:?}", err);
            Default::default()
        }
    };

    let viewers = event.viewer_count;

    // Filter events for the matching viewer minimum amount
    let events = events
       .into_iter()
       .filter(|event| {
           matches!(&event.config.trigger, EventTrigger::ShoutoutReceive { min_viewers } if viewers >= *min_viewers as i64)
       })
       .collect();

    let event_data = EventData {
        input_data: EventInputData::ShoutoutReceive {
            viewer_count: viewers,
        },
        user: Some(TwitchEventUser {
            id: event.user_id,
            name: event.user_name,
            display_name: event.user_display_name,
        }),
    };

    Ok(EventMatchingData {
        events,
        commands: Default::default(),
        event_data,
    })
}

#[cfg(test)]
mod test {
    use super::{
        match_ad_break_event, match_chat_event, match_cheer_bits_event, match_follow_event,
        match_gifted_subscription_event, match_raid_event, match_re_subscription_event,
        match_redeem_event, match_shoutout_receive_event, match_subscription_event,
    };
    use crate::{
        database::{
            entity::{
                commands::{CommandConfig, CommandModel, CommandOutcome, CreateCommand},
                events::{
                    CreateEvent, EventConfig, EventModel, EventOutcome, EventOutcomeSendChat,
                    EventTrigger,
                },
            },
            mock_database,
        },
        events::{
            TwitchEventAdBreakBegin, TwitchEventChatMsg, TwitchEventCheerBits, TwitchEventFollow,
            TwitchEventGiftSub, TwitchEventRaid, TwitchEventReSub, TwitchEventRedeem,
            TwitchEventShoutoutReceive, TwitchEventSub,
        },
    };
    use twitch_api::{
        eventsub::channel::{
            channel_points_custom_reward_redemption::Reward,
            subscription::message::SubscriptionMessage,
        },
        types::{DisplayName, RedemptionId, SubscriptionTier, UserId, UserName},
    };

    /// Tests that a reward redemption event can successfully match using "match_redeem_event"
    /// when the ID is the same as the stored database model
    #[tokio::test]
    async fn test_match_redeem_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Redeem {
                        reward_id: "test-reward".to_string(),
                    },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let reward = serde_json::json!({
            "cost": 0,
            "id": "test-reward",
            "prompt": "",
            "title": "",
        });

        let reward: Reward = serde_json::from_value(reward).unwrap();

        let found_event = match_redeem_event(
            &db,
            TwitchEventRedeem {
                id: RedemptionId::from_static("mock-redemption-id"),
                reward,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
                user_input: "test".to_string(),
            },
        )
        .await
        .expect("missing expected redeem event");

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    /// Tests that no events will be found if the required if "match_redeem_event" has no
    /// matching database events
    #[tokio::test]
    async fn test_match_redeem_event_non_existent() {
        let db = mock_database().await;

        let reward = serde_json::json!({
            "cost": 0,
            "id": "test-reward",
            "prompt": "",
            "title": "",
        });

        let reward: Reward = serde_json::from_value(reward).unwrap();

        let found_event = match_redeem_event(
            &db,
            TwitchEventRedeem {
                id: RedemptionId::from_static("mock-redemption-id"),
                reward,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
                user_input: "test".to_string(),
            },
        )
        .await
        .expect("missing expected redeem event");

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    /// Tests that a cheer bits event can successfully match using "match_cheer_bits_event"
    /// when the ID is the same as the stored database model
    #[tokio::test]
    async fn test_match_cheer_bits_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Bits { min_bits: 0 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_cheer_bits_event(
            &db,
            TwitchEventCheerBits {
                bits: 100,
                anonymous: false,
                user_id: Some(UserId::from_static("mock-user-id")),
                user_name: Some(UserName::from_static("mockuser")),
                user_display_name: Some(DisplayName::from_static("Mock User")),
                message: "test".to_string(),
            },
        )
        .await
        .expect("missing expected redeem event");

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    /// Tests that a cheer bits event will not match using "match_cheer_bits_event"
    /// when the amount of bits is not enough
    #[tokio::test]
    async fn test_match_cheer_bits_event_not_enough_bits() {
        let db = mock_database().await;

        EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Bits { min_bits: 500 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_cheer_bits_event(
            &db,
            TwitchEventCheerBits {
                bits: 100,
                anonymous: false,
                user_id: Some(UserId::from_static("mock-user-id")),
                user_name: Some(UserName::from_static("mockuser")),
                user_display_name: Some(DisplayName::from_static("Mock User")),
                message: "test".to_string(),
            },
        )
        .await
        .expect("missing expected redeem event");

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    /// Tests that no events will be found if the required if "match_cheer_bits_event" has no
    /// matching database events
    #[tokio::test]
    async fn test_match_cheer_bits_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_cheer_bits_event(
            &db,
            TwitchEventCheerBits {
                bits: 100,
                anonymous: false,
                user_id: Some(UserId::from_static("mock-user-id")),
                user_name: Some(UserName::from_static("mockuser")),
                user_display_name: Some(DisplayName::from_static("Mock User")),
                message: "test".to_string(),
            },
        )
        .await
        .expect("missing expected redeem event");

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_follow_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Follow,
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_follow_event(
            &db,
            TwitchEventFollow {
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_follow_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_follow_event(
            &db,
            TwitchEventFollow {
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_subscription_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Subscription,
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_subscription_event(
            &db,
            TwitchEventSub {
                is_gift: false,
                tier: SubscriptionTier::Tier1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_subscription_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_subscription_event(
            &db,
            TwitchEventSub {
                is_gift: false,
                tier: SubscriptionTier::Tier1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_gifted_subscription_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::GiftedSubscription,
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_gifted_subscription_event(
            &db,
            TwitchEventGiftSub {
                anonymous: true,
                total: 1,
                cumulative_total: None,
                tier: SubscriptionTier::Tier1,
                user_id: Some(UserId::from_static("mock-user-id")),
                user_name: Some(UserName::from_static("mockuser")),
                user_display_name: Some(DisplayName::from_static("Mock User")),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_gifted_subscription_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_gifted_subscription_event(
            &db,
            TwitchEventGiftSub {
                anonymous: true,
                total: 1,
                cumulative_total: None,
                tier: SubscriptionTier::Tier1,
                user_id: Some(UserId::from_static("mock-user-id")),
                user_name: Some(UserName::from_static("mockuser")),
                user_display_name: Some(DisplayName::from_static("Mock User")),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_re_subscription_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Subscription,
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let message = serde_json::json!({
            "text": "",
            "emotes": []
        });
        let message: SubscriptionMessage = serde_json::from_value(message).unwrap();

        let found_event = match_re_subscription_event(
            &db,
            TwitchEventReSub {
                cumulative_months: 1,
                duration_months: 1,
                message,
                streak_months: Some(1),
                tier: SubscriptionTier::Tier1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_re_subscription_event_non_existent() {
        let db = mock_database().await;

        let message = serde_json::json!({
            "text": "",
            "emotes": []
        });
        let message: SubscriptionMessage = serde_json::from_value(message).unwrap();

        let found_event = match_re_subscription_event(
            &db,
            TwitchEventReSub {
                cumulative_months: 1,
                duration_months: 1,
                message,
                streak_months: Some(1),
                tier: SubscriptionTier::Tier1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_chat_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Command {
                        message: "!test".to_string(),
                    },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let message = serde_json::json!({
            "text": "!test",
            "fragments": []
        });
        let message: twitch_api::eventsub::channel::chat::Message =
            serde_json::from_value(message).unwrap();

        let found_event = match_chat_event(
            &db,
            TwitchEventChatMsg {
                message_id: "mock-message".into(),
                message,
                cheer: None,

                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_chat_event_command() {
        let db = mock_database().await;

        let expected_command = CommandModel::create(
            &db,
            CreateCommand {
                enabled: true,
                name: "Test Event".to_string(),
                command: "!test".to_string(),
                config: CommandConfig {
                    outcome: CommandOutcome::Template {
                        message: "test".to_string(),
                    },
                    cooldown: Default::default(),
                    require_role: Default::default(),
                },

                aliases: Vec::new(),
            },
        )
        .await
        .unwrap();

        let message = serde_json::json!({
            "text": "!test",
            "fragments": []
        });
        let message: twitch_api::eventsub::channel::chat::Message =
            serde_json::from_value(message).unwrap();

        let found_event = match_chat_event(
            &db,
            TwitchEventChatMsg {
                message_id: "mock-message".into(),
                message,
                cheer: None,

                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let command = found_event
            .commands
            .first()
            .expect("missing matching event");

        // Expect found event to match created
        assert_eq!(command.command.id, expected_command.id);
    }

    #[tokio::test]
    async fn test_match_chat_event_command_not_matching() {
        let db = mock_database().await;

        CommandModel::create(
            &db,
            CreateCommand {
                enabled: true,
                name: "Test Event".to_string(),
                command: "!test".to_string(),
                config: CommandConfig {
                    outcome: CommandOutcome::Template {
                        message: "test".to_string(),
                    },
                    cooldown: Default::default(),
                    require_role: Default::default(),
                },
                aliases: Vec::new(),
            },
        )
        .await
        .unwrap();

        let message = serde_json::json!({
            "text": "!tes",
            "fragments": []
        });
        let message: twitch_api::eventsub::channel::chat::Message =
            serde_json::from_value(message).unwrap();

        let found_event = match_chat_event(
            &db,
            TwitchEventChatMsg {
                message_id: "mock-message".into(),
                message,
                cheer: None,

                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(
            found_event.commands.is_empty(),
            "should not match any events"
        );
    }

    #[tokio::test]
    async fn test_match_chat_event_not_matching() {
        let db = mock_database().await;

        EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Command {
                        message: "!test".to_string(),
                    },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let message = serde_json::json!({
            "text": "",
            "fragments": []
        });
        let message: twitch_api::eventsub::channel::chat::Message =
            serde_json::from_value(message).unwrap();

        let found_event = match_chat_event(
            &db,
            TwitchEventChatMsg {
                message_id: "mock-message".into(),
                message,
                cheer: None,

                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_chat_event_non_existent() {
        let db = mock_database().await;

        let message = serde_json::json!({
            "text": "",
            "fragments": []
        });
        let message: twitch_api::eventsub::channel::chat::Message =
            serde_json::from_value(message).unwrap();

        let found_event = match_chat_event(
            &db,
            TwitchEventChatMsg {
                message_id: "mock-message".into(),
                message,
                cheer: None,

                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_raid_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Raid { min_raiders: 1 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_raid_event(
            &db,
            TwitchEventRaid {
                viewers: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_raid_event_not_enough_raiders() {
        let db = mock_database().await;

        EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::Raid { min_raiders: 5 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_raid_event(
            &db,
            TwitchEventRaid {
                viewers: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_raid_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_raid_event(
            &db,
            TwitchEventRaid {
                viewers: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_ad_break_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::AdBreakBegin,
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_ad_break_event(
            &db,
            TwitchEventAdBreakBegin {
                duration_seconds: 1,
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_ad_break_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_ad_break_event(
            &db,
            TwitchEventAdBreakBegin {
                duration_seconds: 1,
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_shoutout_receive_event() {
        let db = mock_database().await;

        let expected_event = EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::ShoutoutReceive { min_viewers: 1 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_shoutout_receive_event(
            &db,
            TwitchEventShoutoutReceive {
                viewer_count: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        let event = found_event.events.first().expect("missing matching event");

        // Expect found event to match created
        assert_eq!(event.id, expected_event.id);
    }

    #[tokio::test]
    async fn test_match_shoutout_receive_event_not_enough_viewers() {
        let db = mock_database().await;

        EventModel::create(
            &db,
            CreateEvent {
                enabled: true,
                name: "Test Event".to_string(),
                config: EventConfig {
                    trigger: EventTrigger::ShoutoutReceive { min_viewers: 5 },
                    cooldown: Default::default(),
                    outcome: EventOutcome::SendChatMessage(EventOutcomeSendChat {
                        template: "test".to_string(),
                    }),
                    outcome_delay: Default::default(),
                    require_role: Default::default(),
                },
            },
        )
        .await
        .unwrap();

        let found_event = match_shoutout_receive_event(
            &db,
            TwitchEventShoutoutReceive {
                viewer_count: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }

    #[tokio::test]
    async fn test_match_shoutout_receive_event_non_existent() {
        let db = mock_database().await;

        let found_event = match_shoutout_receive_event(
            &db,
            TwitchEventShoutoutReceive {
                viewer_count: 1,
                user_id: UserId::from_static("mock-user-id"),
                user_name: UserName::from_static("mockuser"),
                user_display_name: DisplayName::from_static("Mock User"),
            },
        )
        .await
        .unwrap();

        assert!(found_event.events.is_empty(), "should not match any events");
    }
}
