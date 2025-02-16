use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use twitch_api::{
    eventsub::{
        self,
        channel::{
            channel_points_custom_reward_redemption::Reward, chat::message::Cheer,
            subscription::message::SubscriptionMessage,
        },
    },
    types::{DisplayName, MsgId, RedemptionId, SubscriptionTier, UserId, UserName},
};
use uuid::Uuid;

pub mod matching;
pub mod outcome;
pub mod processing;
pub mod scheduler;

pub type AppEventReceiver = mpsc::Receiver<AppEvent>;
pub type AppEventSender = mpsc::Sender<AppEvent>;

/// Events that can occur within the app
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Twitch redeem event
    Redeem(TwitchEventRedeem),
    /// Twitch bits cheering event
    CheerBits(TwitchEventCheerBits),
    /// Twitch follow event
    Follow(TwitchEventFollow),
    /// Twitch subscription event
    Sub(TwitchEventSub),
    /// Twitch gifted subscription event
    GiftSub(TwitchEventGiftSub),
    /// Twitch resubscription event
    ResubMsg(TwitchEventReSub),
    /// Twitch chat event
    ChatMsg(TwitchEventChatMsg),
    /// Twitch raid event
    Raid(TwitchEventRaid),
    /// Twitch ad break start event
    AdBreakBegin(TwitchEventAdBreakBegin),
    /// Twitch shoutout received
    ShoutoutReceive(TwitchEventShoutoutReceive),
    /// Scheduler timer completed
    TimerCompleted(TimerCompleted),

    /// List of twitch moderators has changed
    ModeratorsChanged,
    /// List of twitch VIPs has changed
    VipsChanged,
    /// Twitch redeem reward data has changed
    RewardsChanged,

    /// Twitch client is resetting (Lost connection/authentication)
    TwitchClientReset,
}

#[derive(Debug, Clone)]
pub struct TimerCompleted {
    pub event_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitchUser {
    pub id: UserId,
    pub name: UserName,
    #[serde(alias = "display_name")]
    pub display_name: DisplayName,
    #[serde(alias = "profile_image_url")]
    pub profile_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TwitchEventUser {
    pub id: UserId,
    pub name: UserName,
    #[serde(alias = "display_name")]
    pub display_name: DisplayName,
}

#[derive(Debug, Clone)]
pub struct TwitchEventRedeem {
    pub id: RedemptionId,
    pub reward: Reward,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
    pub user_input: String,
}

#[derive(Debug, Clone)]
pub struct TwitchEventCheerBits {
    // Total bits gifted
    pub bits: i64,
    pub anonymous: bool,

    // User details empty when cheer is anonymous
    pub user_id: Option<UserId>,
    pub user_name: Option<UserName>,
    pub user_display_name: Option<DisplayName>,

    // Message attached with the bits
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TwitchEventFollow {
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
}

#[derive(Debug, Clone)]
pub struct TwitchEventSub {
    pub is_gift: bool,
    pub tier: SubscriptionTier,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
}

#[derive(Debug, Clone)]
pub struct TwitchEventGiftSub {
    pub anonymous: bool,

    // Total subs gifted
    pub total: i64,

    // Total gifts user has given (If not anonymous)
    pub cumulative_total: Option<i64>,
    pub tier: SubscriptionTier,

    // User details empty when cheer is anonymous
    pub user_id: Option<UserId>,
    pub user_name: Option<UserName>,
    pub user_display_name: Option<DisplayName>,
}

#[derive(Debug, Clone)]
pub struct TwitchEventReSub {
    pub cumulative_months: i64,
    pub duration_months: i64,
    pub message: SubscriptionMessage,
    pub streak_months: Option<i64>,
    pub tier: SubscriptionTier,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
}

#[derive(Debug, Clone)]
pub struct TwitchEventChatMsg {
    pub message_id: MsgId,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
    pub message: eventsub::channel::chat::Message,
    pub cheer: Option<Cheer>,
}

#[derive(Debug, Clone)]
pub struct TwitchEventRaid {
    /// The broadcaster ID that created the raid.
    pub user_id: UserId,
    /// The broadcaster user name that created the raid.
    pub user_name: UserName,
    /// The broadcaster display name that created the raid.
    pub user_display_name: DisplayName,
    /// The number of viewers in the raid.
    pub viewers: i64,
}

#[derive(Debug, Clone)]
pub struct TwitchEventAdBreakBegin {
    /// Duration in seconds of the AD
    pub duration_seconds: i32,
}

#[derive(Debug, Clone)]
pub struct TwitchEventShoutoutReceive {
    /// The broadcaster ID that gave the shoutout
    pub user_id: UserId,
    /// The broadcaster user name that gave the shoutout
    pub user_name: UserName,
    /// The broadcaster display name that gave the shoutout
    pub user_display_name: DisplayName,
    /// The number of users that were watching the from-broadcaster’s stream at the time of the Shoutout.
    pub viewer_count: i64,
}
