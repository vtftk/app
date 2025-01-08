use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitchUser {
    pub id: UserId,
    pub name: UserName,
    pub display_name: DisplayName,
    pub profile_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(unused)]
pub struct TwitchEventUser {
    pub id: UserId,
    pub name: UserName,
    pub display_name: DisplayName,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct TwitchEventRedeem {
    pub id: RedemptionId,
    pub reward: Reward,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
    pub user_input: String,
}

#[derive(Debug, Clone)]
#[allow(unused)]
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
#[allow(unused)]
pub struct TwitchEventFollow {
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct TwitchEventSub {
    pub is_gift: bool,
    pub tier: SubscriptionTier,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
}

#[derive(Debug, Clone)]
#[allow(unused)]
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
#[allow(unused)]
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
#[allow(unused)]
pub struct TwitchEventChatMsg {
    pub message_id: MsgId,
    pub user_id: UserId,
    pub user_name: UserName,
    pub user_display_name: DisplayName,
    pub message: eventsub::channel::chat::Message,
    pub cheer: Option<Cheer>,
}

#[derive(Debug, Clone)]
#[allow(unused)]
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
#[allow(unused)]
pub struct TwitchEventAdBreakBegin {
    /// Duration in seconds of the AD
    pub duration_seconds: i32,
}

#[derive(Debug, Clone)]
#[allow(unused)]
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

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum TwitchEvent {
    Redeem(TwitchEventRedeem),
    CheerBits(TwitchEventCheerBits),
    Follow(TwitchEventFollow),
    Sub(TwitchEventSub),
    GiftSub(TwitchEventGiftSub),
    ResubMsg(TwitchEventReSub),
    ChatMsg(TwitchEventChatMsg),
    Raid(TwitchEventRaid),
    AdBreakBegin(TwitchEventAdBreakBegin),
    ShoutoutReceive(TwitchEventShoutoutReceive),

    ModeratorsChanged,
    VipsChanged,
    RewardsChanged,

    Reset,
}