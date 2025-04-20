use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServerDetails {
    pub identifier: &'static str,
}

const IDENTIFIER: &str = "VTFTK_SERVER";

/// GET /server/details
///
/// Get simple details about the server, used to check if a server
/// is alive by clients
pub async fn details() -> Json<ServerDetails> {
    Json(ServerDetails {
        identifier: IDENTIFIER,
    })
}
