use super::{
    models::{TwitchEvent, TwitchUser},
    websocket::WebsocketManagedTask,
};
use anyhow::{anyhow, Context};
use futures::TryStreamExt;
use log::error;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::{
    join,
    sync::{broadcast, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use twitch_api::{
    helix::{
        channels::{Follower, GetChannelFollowersRequest, Vip},
        chat::{
            ChannelEmote, SendAShoutoutRequest, SendAShoutoutResponse, SendChatAnnouncementBody,
            SendChatAnnouncementRequest, SendChatAnnouncementResponse, SendChatMessageBody,
            SendChatMessageRequest, SendChatMessageResponse,
        },
        moderation::Moderator,
        points::CustomReward,
        EmptyBody,
    },
    twitch_oauth2::{AccessToken, UserToken},
    types::{MsgId, UserId},
    HelixClient,
};

#[derive(Clone)]
pub struct Twitch {
    _inner: Arc<TwitchInner>,
}

impl Twitch {
    pub fn new(app_handle: AppHandle) -> (Self, broadcast::Receiver<TwitchEvent>) {
        let (tx, rx) = broadcast::channel(10);
        (
            Self {
                _inner: Arc::new(TwitchInner {
                    helix_client: HelixClient::default(),
                    state: Default::default(),
                    tx,
                    app_handle,
                }),
            },
            rx,
        )
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

    pub async fn delete_chat_message(&self, message_id: MsgId) -> anyhow::Result<()> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        self.helix_client()
            .delete_chat_message(user_id.clone(), user_id.clone(), message_id, &token)
            .await?;

        Ok(())
    }

    pub async fn delete_all_chat_messages(&self) -> anyhow::Result<()> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        self.helix_client()
            .delete_all_chat_message(&user_id, &user_id, &token)
            .await?;

        Ok(())
    }

    pub async fn create_stream_marker(&self, description: Option<String>) -> anyhow::Result<()> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        self.helix_client()
            .create_stream_marker(&user_id, description.unwrap_or_default(), &token)
            .await?;

        Ok(())
    }

    pub async fn send_chat_announcement_message(
        &self,
        message: String,
        color: String,
    ) -> anyhow::Result<SendChatAnnouncementResponse> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        // Create chat message request
        let request = SendChatAnnouncementRequest::new(user_id.clone(), user_id.clone());
        let body = SendChatAnnouncementBody::new(message, color.as_str())
            .context("failed to create body")?;

        // Send request and get response
        let response: SendChatAnnouncementResponse = self
            .helix_client()
            .req_post(request, body, &token)
            .await?
            .data;

        Ok(response)
    }

    pub async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<TwitchUser>> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        let user = self
            .helix_client()
            .get_user_from_login(username, &token)
            .await?;

        Ok(user.map(|user| TwitchUser {
            id: user.id,
            name: user.login,
            display_name: user.display_name,
            profile_image_url: user.profile_image_url,
        }))
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

    pub async fn send_shoutout(
        &self,
        target_user_id: UserId,
    ) -> anyhow::Result<SendAShoutoutResponse> {
        // Obtain twitch access token
        let token = self.get_user_token().await.context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        // Create chat message request
        let request = SendAShoutoutRequest::new(user_id.clone(), target_user_id, user_id.clone());

        // Send request and get response
        let response: SendAShoutoutResponse = self
            .helix_client()
            .req_post(request, EmptyBody, &token)
            .await?
            .data;

        Ok(response)
    }

    pub async fn get_user_token(&self) -> Option<UserToken> {
        let lock = &*self.state().await;
        match lock {
            TwitchManagerState::Initial => None,
            TwitchManagerState::Authenticated(state) => Some(state.token.clone()),
        }
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
    tx: broadcast::Sender<TwitchEvent>,
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
