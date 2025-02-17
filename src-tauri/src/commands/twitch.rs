use crate::commands::CmdResult;
use crate::database::entity::app_data::AppDataModel;
use crate::database::entity::secrets::SecretsModel;
use crate::database::entity::TWITCH_SECRET_KEY;
use crate::database::DbPool;
use crate::twitch::manager::Twitch;
use anyhow::Context;
use reqwest::Url;
use std::sync::Arc;
use tauri::State;
use twitch_api::helix::points::CustomReward;

/// Requests the list of available redeems from the broadcasters channel.
///
/// Used on the frontend for the dropdown menu that allows you to pick
/// from the list of redeems as an event trigger
#[tauri::command]
pub async fn get_redeems_list(twitch: State<'_, Twitch>) -> CmdResult<Arc<[CustomReward]>> {
    Ok(twitch
        .get_rewards_list()
        .await
        .context("failed to load redeems")?)
}

/// Reloads the list of available redeems
#[tauri::command]
pub async fn refresh_redeems_list(twitch: State<'_, Twitch>) -> CmdResult<()> {
    twitch.load_rewards_list().await?;
    Ok(())
}

/// Obtain a URL for use logging into twitch using OAuth2
#[tauri::command]
pub async fn get_twitch_oauth_uri(
    twitch: State<'_, Twitch>,
    db: tauri::State<'_, DbPool>,
) -> CmdResult<String> {
    let http_port = AppDataModel::get_http_port(db.inner()).await?;

    let redirect_url = format!("http://localhost:{http_port}/oauth",);
    let redirect_url = Url::parse(&redirect_url).context("invalid redirect_uri")?;
    let url = twitch.create_oauth_uri(redirect_url)?;

    Ok(url)
}

#[tauri::command]
pub async fn is_authenticated(twitch: tauri::State<'_, Twitch>) -> CmdResult<bool> {
    Ok(twitch.is_authenticated().await)
}

#[tauri::command]
pub async fn logout(
    twitch: tauri::State<'_, Twitch>,
    db: tauri::State<'_, DbPool>,
) -> CmdResult<()> {
    twitch.reset().await;
    SecretsModel::delete_by_key(db.inner(), TWITCH_SECRET_KEY).await?;

    Ok(())
}
