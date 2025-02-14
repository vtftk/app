use axum::{
    routing::{get, post, put},
    Router,
};

mod calibration;
mod data;
mod oauth;
mod overlay;

pub fn router() -> Router {
    Router::new()
        // OAuth complete page and OAuth complete endpoint
        .route("/oauth", get(oauth::handle_oauth))
        .route("/oauth/complete", post(oauth::handle_oauth_complete))
        // Calibration
        .route(
            "/calibration",
            get(calibration::handle_calibration_data)
                .post(calibration::handle_calibration_progress),
        )
        // App data
        .route("/app-data", get(data::get_app_data))
        // Asset endpoints (Bonks, sounds, defaults, etc...)
        .route("/content/:folder/:name", get(data::get_content_file))
        .route("/defaults/:folder/:name", get(data::get_defaults_file))
        // Overlay endpoints
        .route("/overlay", get(overlay::page))
        .route("/overlay/events", get(overlay::handle_sse))
        .route("/overlay/icon", get(overlay::icon))
        .route("/overlay/data", put(overlay::update_overlay_data))
        // VTube studio access token endpoints
        .route(
            "/data/vt-auth-token",
            get(data::handle_get_auth_token).post(data::handle_set_auth_token),
        )
}
