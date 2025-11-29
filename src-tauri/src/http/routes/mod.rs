use axum::{
    routing::{get, post, put},
    Router,
};

mod calibration;
mod data;
mod items;
mod oauth;
mod overlay;
mod server;
mod sounds;

pub fn router() -> Router {
    Router::new()
        // Get server details
        .route("/server/details", get(server::details))
        // OAuth complete page and OAuth complete endpoint
        .route("/oauth", get(oauth::handle_oauth))
        .route("/oauth/complete", post(oauth::handle_oauth_complete))
        // Calibration
        .route(
            "/calibration",
            get(calibration::handle_calibration_data)
                .post(calibration::handle_calibration_progress),
        )
        // Asset endpoints (Bonks, sounds, defaults, etc...)
        .route("/content/{folder}/{name}", get(data::get_content_file))
        .route("/defaults/{folder}/{name}", get(data::get_defaults_file))
        // Overlay endpoints
        .route("/overlay", get(overlay::page))
        .route("/overlay/config", get(overlay::get_overlay_config))
        .route(
            "/overlay/events",
            get(overlay::handle_sse).post(overlay::emit_event),
        )
        .route("/overlay/icon", get(overlay::icon))
        .route("/overlay/data", put(overlay::update_overlay_data))
        // VTube studio access token endpoints
        .route(
            "/data/vt-auth-token",
            get(data::handle_get_auth_token).post(data::handle_set_auth_token),
        )
        // Requesting items
        .route("/items", get(items::all))
        .route("/items/query-by-name", post(items::query_by_name))
        .route("/items/query-by-id", post(items::query_by_id))
        // Requesting sounds
        .route("/sounds", get(sounds::all))
        .route("/sounds/query-by-name", post(sounds::query_by_name))
        .route("/sounds/query-by-id", post(sounds::query_by_id))
}
