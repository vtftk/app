use super::websocket::WebsocketManagedTask;
use crate::{database::entity::twitch_access::TwitchAccessModel, events::AppEventSender};
use anyhow::{anyhow, Context};
use futures::TryStreamExt;
use log::{debug, error, info};
use sea_orm::{DatabaseConnection, ModelTrait};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::{
    join,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use twitch_api::{
    helix::{
        channels::{Follower, GetChannelFollowersRequest, Vip},
        chat::{
            ChannelEmote, SendChatMessageBody, SendChatMessageRequest, SendChatMessageResponse,
        },
        moderation::Moderator,
        points::CustomReward,
        Scope,
    },
    twitch_oauth2::{types::ClientIdRef, AccessToken, ImplicitUserTokenBuilder, UserToken},
    types::UserId,
    HelixClient,
};

/// If you are forking this app program for your own use, please create your own
/// twitch developer application client ID at https://dev.twitch.tv/console/apps
pub const TWITCH_CLIENT_ID: &ClientIdRef =
    ClientIdRef::from_static("x0zzeitiwvgblu743qnxzaipa9e01z");

/// Scopes required from twitch by the app
pub const TWITCH_REQUIRED_SCOPES: &[Scope] = &[
    // View live Stream Chat and Rooms messages
    Scope::UserReadChat,
    // View Channel Points rewards and their redemptions on your channel.
    Scope::ChannelReadRedemptions,
    // Get a list of all subscribers to your channel and check if a user is subscribed to your channel
    Scope::ChannelReadSubscriptions,
    // View your channel's Bits information
    Scope::BitsRead,
    // Read the list of followers in channels where you are a moderator.
    // (Followers list & Follower event sub)
    Scope::ModeratorReadFollowers,
    // View a channel’s moderation data including Moderators, Bans, Timeouts, and Automod settings.
    // (Moderators list & Moderator event sub)
    Scope::ModerationRead,
    // Read the list of VIPs in your channel.
    // (Vip list and VIP event sub)
    Scope::ChannelReadVips,
    // Send chat messages
    Scope::UserWriteChat,
    // Allows sending shoutouts from the scripting API
    Scope::ModeratorManageShoutouts,
    // Allow sending chat announcements
    Scope::ModeratorManageAnnouncements,
    // Allow deleting messages
    Scope::ModeratorManageChatMessages,
    // Allow creating stream markers
    Scope::ChannelManageBroadcast,
    // Scope to read ad break messages
    Scope::ChannelReadAds,
];

#[derive(Clone)]
pub struct Twitch {
    _inner: Arc<TwitchInner>,
}

impl Twitch {
    pub fn new(app_handle: AppHandle, tx: AppEventSender) -> Self {
        Self {
            _inner: Arc::new(TwitchInner {
                helix_client: HelixClient::default(),
                state: Default::default(),
                tx,
                app_handle,
            }),
        }
    }

    pub fn create_oauth_uri(&self, redirect_url: reqwest::Url) -> anyhow::Result<String> {
        let (url, _csrf) = ImplicitUserTokenBuilder::new(TWITCH_CLIENT_ID.into(), redirect_url)
            .set_scopes(TWITCH_REQUIRED_SCOPES.to_vec())
            .generate_url();

        Ok(url.to_string())
    }

    /// Attempts to authenticate with twitch using an existing access token (From the database)
    pub async fn attempt_auth_stored(&self, db: DatabaseConnection) {
        let access = match TwitchAccessModel::get(&db).await {
            Ok(Some(value)) => value,
            Ok(None) => {
                debug!("not authenticated, skipping login");
                return;
            }
            Err(err) => {
                error!("failed to load twitch access: {err:?}");
                return;
            }
        };

        let access_token = access.access_token.0.clone();
        let scopes = &access.scopes.0;

        for required_scope in TWITCH_REQUIRED_SCOPES {
            if !scopes.contains(required_scope) {
                info!("logging out current access token, missing required scope");

                // Clear outdated / invalid access token
                _ = access.delete(&db).await;
                return;
            }
        }

        if let Err(err) = self.attempt_auth_existing_token(access_token).await {
            error!("stored access token is invalid: {}", err);

            // Clear outdated / invalid access token
            _ = access.delete(&db).await;
        }
    }

    pub async fn attempt_auth_existing_token(
        &self,
        access_token: AccessToken,
    ) -> anyhow::Result<()> {
        // Create user token (Validates it with the twitch backend)
        let user_token = self.create_user_token(access_token).await?;
        self.set_authenticated(user_token).await;

        Ok(())
    }

    pub async fn is_authenticated(&self) -> bool {
        let lock = &*self.state().await;
        matches!(lock, TwitchManagerState::Authenticated { .. })
    }

    pub async fn send_chat_message(
        &self,
        message: &str,
    ) -> anyhow::Result<SendChatMessageResponse> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        // Create chat message request
        let request = SendChatMessageRequest::new();
        let body = SendChatMessageBody::new(user_id.clone(), user_id, message);

        // Send request and get response
        let response: SendChatMessageResponse = self
            .helix_client()
            .req_post(request, body, &token)
            .await?
            .data;

        Ok(response)
    }

    pub async fn get_channel_emotes(&self, user_id: UserId) -> anyhow::Result<Vec<ChannelEmote>> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        let emotes = self
            .helix_client()
            .get_channel_emotes_from_id(user_id, &token)
            .await?;

        Ok(emotes)
    }

    pub async fn get_follower_by_id(&self, user_id: UserId) -> anyhow::Result<Option<Follower>> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let broadcaster_id = token.user_id.clone();

        // Create chat message request
        let request = GetChannelFollowersRequest::broadcaster_id(broadcaster_id).user_id(user_id);

        // Send request and get response
        let mut response: Vec<Follower> = self.helix_client().req_get(request, &token).await?.data;

        Ok(response.pop())
    }

    pub async fn get_user_token(&self) -> Option<UserToken> {
        let lock = &*self.state().await;
        match lock {
            TwitchManagerState::Initial => None,
            TwitchManagerState::Authenticated(state) => Some(state.token.clone()),
        }
    }

    pub async fn get_user_id(&self) -> Option<UserId> {
        self.get_user_token().await.map(|token| token.user_id)
    }

    pub async fn set_authenticated(&self, token: UserToken) {
        {
            let lock = &mut *self.state_mut().await;

            let websocket = WebsocketManagedTask::create(
                self.helix_client().clone(),
                self._inner.tx.clone(),
                token.clone(),
            );

            *lock = TwitchManagerState::Authenticated(TwitchManagerStateAuthenticated {
                token,
                _websocket: websocket,
                moderators: None,
                vips: None,
                rewards: None,
            });
        }

        // Tell the app we are authenticated
        _ = self._inner.app_handle.emit("authenticated", ());

        // Load initial moderator and VIP lists
        let (rewards_result, vips_result, mods_result) = join!(
            self.load_rewards_list(),
            self.load_vip_list(),
            self.load_moderator_list()
        );

        if let Err(err) = rewards_result {
            error!("failed to load rewards: {:?}", err);
        }

        if let Err(err) = vips_result {
            error!("failed to load vips: {:?}", err);
        }

        if let Err(err) = mods_result {
            error!("failed to load mods: {:?}", err);
        }
    }

    pub async fn reset(&self) {
        {
            let lock = &mut *self.state_mut().await;
            *lock = TwitchManagerState::Initial;
        }

        // Tell the app we are authenticated
        _ = self._inner.app_handle.emit("logout", ());
    }

    pub async fn get_moderator_list(&self) -> anyhow::Result<Arc<[Moderator]>> {
        // First attempt to read existing list
        {
            let state = &*self.state().await;
            match state {
                TwitchManagerState::Initial => return Err(anyhow!("not authenticated")),
                TwitchManagerState::Authenticated(state) => {
                    if let Some(moderators) = state.moderators.as_ref() {
                        return Ok(moderators.clone());
                    }
                }
            }
        }

        let moderators = self.request_moderator_list().await?;
        let moderators: Arc<[Moderator]> = moderators.into();

        // Write new list
        let state = &mut *self.state_mut().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                state.moderators = Some(moderators.clone());
                Ok(moderators)
            }
        }
    }

    pub async fn get_vip_list(&self) -> anyhow::Result<Arc<[Vip]>> {
        // First attempt to read existing list
        {
            let state = &*self.state().await;
            match state {
                TwitchManagerState::Initial => return Err(anyhow!("not authenticated")),
                TwitchManagerState::Authenticated(state) => {
                    if let Some(vips) = state.vips.as_ref() {
                        return Ok(vips.clone());
                    }
                }
            }
        }
        let vips = self.request_vip_list().await?;

        // Write new list
        let state = &mut *self.state_mut().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                let vips: Arc<[Vip]> = vips.into();
                state.vips = Some(vips.clone());

                Ok(vips)
            }
        }
    }

    pub async fn get_rewards_list(&self) -> anyhow::Result<Arc<[CustomReward]>> {
        let state = &*self.state().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                if let Some(rewards) = state.rewards.as_ref() {
                    Ok(rewards.clone())
                } else {
                    Err(anyhow!(""))
                }
            }
        }
    }

    pub async fn load_moderator_list(&self) -> anyhow::Result<()> {
        let moderators = self.request_moderator_list().await?;
        let moderators: Arc<[Moderator]> = moderators.into();

        // Write new list
        let state = &mut *self.state_mut().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                state.moderators = Some(moderators);
                Ok(())
            }
        }
    }

    pub async fn load_vip_list(&self) -> anyhow::Result<()> {
        let vips = self.request_vip_list().await?;
        let vips: Arc<[Vip]> = vips.into();

        // Write new list
        let state = &mut *self.state_mut().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                state.vips = Some(vips);
                Ok(())
            }
        }
    }

    pub async fn load_rewards_list(&self) -> anyhow::Result<()> {
        let rewards = self.request_rewards_list().await?;
        let rewards: Arc<[CustomReward]> = rewards.into();

        // Write new list
        let state = &mut *self.state_mut().await;
        match state {
            TwitchManagerState::Initial => Err(anyhow!("not authenticated")),
            TwitchManagerState::Authenticated(state) => {
                state.rewards = Some(rewards);
                Ok(())
            }
        }
    }

    async fn request_moderator_list(&self) -> anyhow::Result<Vec<Moderator>> {
        let user_token = self.get_user_token().await.context("not authenticated")?;
        let user_id = user_token.user_id.clone();

        let moderators: Vec<Moderator> = self
            .helix_client()
            .get_moderators_in_channel_from_id(user_id, &user_token)
            .try_collect()
            .await?;

        Ok(moderators)
    }

    async fn request_vip_list(&self) -> anyhow::Result<Vec<Vip>> {
        let user_token = self.get_user_token().await.context("not authenticated")?;
        let user_id = user_token.user_id.clone();

        let moderators: Vec<Vip> = self
            .helix_client()
            .get_vips_in_channel(user_id, &user_token)
            .try_collect()
            .await?;

        Ok(moderators)
    }

    async fn request_rewards_list(&self) -> anyhow::Result<Vec<CustomReward>> {
        let user_token = self.get_user_token().await.context("not authenticated")?;
        let user_id = user_token.user_id.clone();
        let rewards = self
            .helix_client()
            .get_all_custom_rewards(user_id, false, &user_token)
            .await?;

        Ok(rewards)
    }

    pub async fn create_user_token(&self, access_token: AccessToken) -> anyhow::Result<UserToken> {
        UserToken::from_existing(self.helix_client(), access_token, None, None)
            .await
            .context("failed to create user token")
    }

    #[inline]
    async fn state(&self) -> RwLockReadGuard<'_, TwitchManagerState> {
        self._inner.state.read().await
    }

    #[inline]
    async fn state_mut(&self) -> RwLockWriteGuard<'_, TwitchManagerState> {
        self._inner.state.write().await
    }

    #[inline]
    fn helix_client(&self) -> &HelixClient<'static, reqwest::Client> {
        &self._inner.helix_client
    }
}

struct TwitchInner {
    helix_client: HelixClient<'static, reqwest::Client>,
    state: RwLock<TwitchManagerState>,
    tx: AppEventSender,
    app_handle: AppHandle,
}

pub struct TwitchManagerStateAuthenticated {
    /// Token for the authenticated user
    token: UserToken,

    /// Currently active websocket connection
    _websocket: WebsocketManagedTask,

    /// List of available rewards
    rewards: Option<Arc<[CustomReward]>>,

    /// Current loaded list of moderators
    moderators: Option<Arc<[Moderator]>>,
    /// Current loaded list of vips
    vips: Option<Arc<[Vip]>>,
}

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
enum TwitchManagerState {
    // Twitch is not yet authenticated
    #[default]
    Initial,
    // Twitch is authenticated
    Authenticated(TwitchManagerStateAuthenticated),
}
