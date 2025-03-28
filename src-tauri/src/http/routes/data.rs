use crate::{
    database::{
        entity::{
            secrets::{SecretsModel, SetSecret},
            VT_SECRET_KEY,
        },
        DbPool,
    },
    http::{
        error::{DynHttpError, HttpResult},
        models::{GetAuthTokenResponse, SetAuthTokenRequest},
    },
    storage::Storage,
};
use anyhow::Context;
use axum::{
    body::Body,
    extract::Path,
    http::{Response, StatusCode},
    Extension, Json,
};
use reqwest::header::{CACHE_CONTROL, CONTENT_TYPE};
use tauri::{path::BaseDirectory, AppHandle, Manager};

/// GET /content/:folder/:name  
///
/// Retrieve the contents of a file from one of the content folders
pub async fn get_content_file(
    Path((folder, name)): Path<(String, String)>,
    Extension(storage): Extension<Storage>,
) -> Result<Response<Body>, DynHttpError> {
    let file = storage.get_file(folder, name).await?;
    let file = match file {
        Some(file) => file,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(vec![].into())
                .context("failed to make response")?)
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, file.mime.essence_str())
        .header(CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(file.content.into())
        .context("failed to make response")?)
}

/// GET /defaults/:folder/:name
pub async fn get_defaults_file(
    Path((folder, name)): Path<(String, String)>,
    Extension(app): Extension<AppHandle>,
) -> Result<Response<Body>, DynHttpError> {
    let file_path = app
        .path()
        .resolve(format!("defaults/{folder}/{name}"), BaseDirectory::Resource)
        .context("failed to get file path")?;

    if !file_path.exists() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(vec![].into())
            .context("failed to make response")?);
    }

    let mime = mime_guess::from_path(&file_path);

    let file_bytes = tokio::fs::read(file_path)
        .await
        .context("failed to read content file")?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, mime.first_or_octet_stream().essence_str())
        .header(CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(file_bytes.into())
        .context("failed to make response")?)
}

/// POST /data/vt-auth-token
///
/// Set the current VTube Studio access token for the overlay
pub async fn handle_set_auth_token(
    Extension(db): Extension<DbPool>,
    Json(req): Json<SetAuthTokenRequest>,
) -> HttpResult<()> {
    if let Some(access_token) = req.auth_token {
        // Set new access token
        SecretsModel::set(
            &db,
            SetSecret {
                key: VT_SECRET_KEY.to_string(),
                value: access_token,
                metadata: serde_json::Value::Null,
            },
        )
        .await
        .context("failed to store vt access token")?;
    } else {
        // Clear existing access token
        SecretsModel::delete_by_key(&db, VT_SECRET_KEY)
            .await
            .context("failed to delete original token")?;
    }

    Ok(Json(()))
}

/// GET /data/vt-auth-token
///
/// Retrieve the current VTube Studio access token for the overlay
pub async fn handle_get_auth_token(
    Extension(db): Extension<DbPool>,
) -> HttpResult<GetAuthTokenResponse> {
    let secret = SecretsModel::get_by_key(&db, VT_SECRET_KEY)
        .await
        .context("failed to get access")?;

    Ok(Json(GetAuthTokenResponse {
        auth_token: secret.map(|access| access.value),
    }))
}
