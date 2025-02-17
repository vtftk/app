pub mod app_data;
pub mod chat_history;
pub mod command_execution;
pub mod command_log;
pub mod commands;
pub mod event_execution;
pub mod event_log;
pub mod events;
pub mod items;
pub mod key_value;
pub mod model_data;
pub mod secrets;
pub mod shared;
pub mod sounds;

pub const TWITCH_SECRET_KEY: &str = "__TWITCH_SECRET__";
pub const VT_SECRET_KEY: &str = "__VT_STUDIO_SECRET__";
