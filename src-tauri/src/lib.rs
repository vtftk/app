use anyhow::Context;
use database::{clean_old_data, entity::app_data::AppDataModel};
use events::{
    create_event_channel, processing::process_twitch_events, scheduler::create_scheduler,
};
use script::{events::ScriptEventActor, runtime::create_script_executor};
use sea_orm::DatabaseConnection;
use state::runtime_app_data::RuntimeAppDataStore;
use std::error::Error;
use tauri::{
    async_runtime::{block_on, spawn},
    App, AppHandle, Manager, RunEvent,
};
use twitch::manager::Twitch;

mod commands;
mod database;
mod events;
mod http;
mod integrations;
mod script;
mod state;
mod tray;
mod twitch;

/// Prevent slow changes from macro by using a separate entrypoint
/// from the macro
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        // Shell access plugin
        .plugin(tauri_plugin_shell::init())
        // Don't allow creation of multiple windows, instead focus the existing window
        .plugin(tauri_plugin_single_instance::init(
            handle_duplicate_instance,
        ))
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            // Calibration commands
            commands::calibration::set_calibration_step,
            commands::calibration::calibration_move_model,
            commands::calibration::get_calibration_data,
            // Testing and running commands
            commands::test::test_throw,
            commands::test::test_throw_barrage,
            commands::test::test_sound,
            commands::test::detect_vtube_studio,
            // Data manipulation comments
            commands::data::get_app_data,
            commands::data::get_runtime_app_data,
            commands::data::set_app_data,
            commands::data::upload_file,
            commands::data::update_hotkeys,
            commands::data::get_overlay_url,
            // Twitch commands
            commands::twitch::get_twitch_oauth_uri,
            commands::twitch::is_authenticated,
            commands::twitch::logout,
            commands::twitch::get_redeems_list,
            commands::twitch::refresh_redeems_list,
            // Item manipulation commands
            commands::items::get_item_by_id,
            commands::items::get_items,
            commands::items::get_item_sounds,
            commands::items::create_item,
            commands::items::update_item,
            commands::items::update_item_orderings,
            commands::items::delete_item,
            commands::items::append_item_impact_sounds,
            // Sound commands
            commands::sounds::get_sounds,
            commands::sounds::get_sound_by_id,
            commands::sounds::create_sound,
            commands::sounds::update_sound,
            commands::sounds::delete_sound,
            commands::sounds::update_sound_orderings,
            // Command commands
            commands::commands::get_commands,
            commands::commands::get_command_by_id,
            commands::commands::create_command,
            commands::commands::update_command,
            commands::commands::delete_command,
            commands::commands::get_command_logs,
            commands::commands::delete_command_logs,
            commands::commands::update_command_orderings,
            commands::commands::get_command_executions,
            commands::commands::delete_command_executions,
            // Event commands
            commands::events::get_events,
            commands::events::get_event_by_id,
            commands::events::create_event,
            commands::events::update_event,
            commands::events::delete_event,
            commands::events::test_event_by_id,
            commands::events::update_event_orderings,
            commands::events::get_event_executions,
            commands::events::delete_event_executions,
            commands::events::get_event_logs,
            commands::events::delete_event_logs,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        // Prevent default exit handling, app exiting is done
        .run(handle_app_event);
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let handle = app.handle();

    let app_data_path = app
        .path()
        .app_data_dir()
        .context("failed to get app data dir")?;

    let db = block_on(database::connect_database(app_data_path.join("app.db")))
        .context("failed to load database")?;

    let (twitch, twitch_event_rx) = Twitch::new(handle.clone());
    let (event_tx, event_rx) = create_event_channel();

    let runtime_app_data = RuntimeAppDataStore::new(handle.clone());

    let script_handle = create_script_executor(app_data_path.join("modules"));

    // Create background event scheduler
    let scheduler_handle = create_scheduler(
        db.clone(),
        twitch.clone(),
        script_handle.clone(),
        event_tx.clone(),
    );

    // Add auto updater plugin if auto updating is allowed
    block_on(setup_auto_update(&db, handle))?;

    // Run background cleanup
    spawn(clean_old_data(db.clone()));

    // Provide runtime app data stores
    app.manage(runtime_app_data.clone());

    // Provide access to the scheduler
    app.manage(scheduler_handle);

    // Provide access to twitch manager and event sender
    app.manage(event_tx.clone());
    app.manage(twitch.clone());

    // Provide access to script running and
    app.manage(script_handle.clone());

    // Provide database access
    app.manage(db.clone());

    // Attempt to authenticate with twitch using the saved token
    _ = spawn({
        let twitch = twitch.clone();
        let db = db.clone();

        async move { twitch.attempt_auth_stored(db).await }
    });

    // Initialize script actor
    let actor = ScriptEventActor::new(event_tx.clone(), db.clone(), twitch.clone());
    block_on(script::events::init_global_script_event_actor(actor));

    // Handle events triggered by twitch
    _ = spawn(process_twitch_events(
        db.clone(),
        twitch.clone(),
        script_handle,
        event_tx.clone(),
        twitch_event_rx,
    ));

    // Run HTTP server
    _ = spawn(http::start_http_server(
        db,
        event_rx,
        handle.clone(),
        twitch,
        runtime_app_data,
    ));

    tray::create_tray_menu(app)?;

    Ok(())
}

/// Handle initialization of a second app instance, focuses the main
/// window instead of allowing multiple instances
fn handle_duplicate_instance(app: &AppHandle, _args: Vec<String>, _cwd: String) {
    let _ = app
        .get_webview_window("main")
        .expect("no main window")
        .set_focus();
}

/// Handles app events, used for the minimize to tray event
fn handle_app_event(app: &AppHandle, event: RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
        let db = app.state::<DatabaseConnection>();
        let main_config = block_on(AppDataModel::get_main_config(db.inner()));
        let minimize_to_tray = main_config.is_ok_and(|value| value.minimize_to_tray);

        if code.is_none() && minimize_to_tray {
            api.prevent_exit();
        }
    }
}

/// Initialize auto updating
async fn setup_auto_update(db: &DatabaseConnection, app_handle: &AppHandle) -> anyhow::Result<()> {
    let auto_updating = AppDataModel::is_auto_updating(db).await?;
    if auto_updating {
        app_handle.plugin(tauri_plugin_updater::Builder::new().build())?;
    }

    Ok(())
}
