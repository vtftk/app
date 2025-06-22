use crate::{script::runtime::ScriptRuntimeDataExt, twitch::manager::TWITCH_CLIENT_ID};
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};
use twitch_api::{
    twitch_oauth2::{AccessToken, ClientId},
    types::UserId,
};

#[derive(Serialize)]
pub struct TwitchCredentials {
    pub token: AccessToken,
    pub user_id: UserId,
    pub client_id: ClientId,
}

/// Get twitch credentials within JS
#[op2(async)]
#[serde]
pub async fn op_twitch_get_credentials(
    state: Rc<RefCell<OpState>>,
) -> Result<Option<TwitchCredentials>, JsErrorBox> {
    let twitch = state.twitch()?;
    let token = match twitch.get_user_token() {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(TwitchCredentials {
        token: token.access_token,
        user_id: token.user_id,
        client_id: TWITCH_CLIENT_ID.to_owned(),
    }))
}
