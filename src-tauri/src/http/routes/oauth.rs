use crate::{
    database::entity::{
        secrets::{SecretModel, SetSecret},
        TWITCH_SECRET_KEY,
    },
    http::error::HttpResult,
    twitch::manager::Twitch,
};
use anyhow::Context;
use axum::{response::IntoResponse, Extension, Json};
use reqwest::header::CONTENT_TYPE;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use twitch_api::{helix::Scope, twitch_oauth2::AccessToken};

/// Embedded oauth response page for handling sending the fragment
static OAUTH_RESPONSE_PAGE: &str = include_str!("../resources/twitch-oauth-response.html");

/// GET /oauth
///
/// Handles an OAuth response from twitch
///
/// Web server does not support handling fragments so we send back a small
/// HTML page which sends us the token to
pub async fn handle_oauth() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/html")], OAUTH_RESPONSE_PAGE)
}

#[derive(Debug, Deserialize)]
pub struct OAuthComplete {
    access_token: AccessToken,
    scopes: Vec<Scope>,
}

/// POST /oauth/complete
///
/// Handles the completion of OAuth logging into the twitch account storing
/// the access token and authorized scopes
pub async fn handle_oauth_complete(
    Extension(db): Extension<DatabaseConnection>,
    Extension(twitch): Extension<Twitch>,
    Json(req): Json<OAuthComplete>,
) -> HttpResult<()> {
    let token = twitch.create_user_token(req.access_token).await?;

    let access_token = token.access_token.clone();
    let scopes = req.scopes;

    twitch.set_authenticated(token).await;

    // Set new access token
    SecretModel::set(
        &db,
        SetSecret {
            key: TWITCH_SECRET_KEY.to_string(),
            value: access_token.secret().to_string(),
            metadata: serde_json::to_value(scopes).context("failed to encode tokens")?,
        },
    )
    .await
    .context("failed to store access token")?;

    Ok(Json(()))
}
