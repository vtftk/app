use anyhow::Context;
use commands::events::update_scheduler_events;
use database::{clean_old_data, entity::app_data::AppDataModel, DbPool};
use events::{processing::process_events, scheduler::create_scheduler};
use http::{create_http_socket, HttpExtensions, ServerPort};
use log::error;
use overlay::{create_overlay_channel, OverlayDataStore};
use script::runtime::{create_script_executor, ScriptRuntimeData};
use std::error::Error;
use storage::Storage;
use tauri::{
    async_runtime::{block_on, spawn},
    App, AppHandle, Manager, RunEvent,
};
use tokio::sync::mpsc;
use twitch::manager::Twitch;

mod commands;
mod database;
mod events;
mod export;
mod http;
mod overlay;
mod script;
mod storage;
mod tray;
mod twitch;

/// Prevent slow changes from macro by using a separate entrypoint
/// from the macro
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    use crate::commands::{calibration, commands, data, events, items, sounds, test, twitch};

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // Don't allow creation of multiple windows, instead focus the existing window
        .plugin(tauri_plugin_single_instance::init(
            handle_duplicate_instance,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            // Calibration commands
            calibration::set_calibration_step,
            calibration::calibration_move_model,
            calibration::get_calibration_data,
            // Testing and running commands
            test::test_throw,
            test::test_throw_barrage,
            test::detect_vtube_studio,
            // Data manipulation comments
            data::get_app_data,
            data::get_runtime_app_data,
            data::set_app_data,
            data::upload_file,
            data::update_hotkeys,
            data::get_overlay_url,
            data::get_chat_history_estimate_size,
            data::get_executions_estimate_size,
            data::get_logs_estimate_size,
            data::get_http_port,
            // Twitch commands
            twitch::get_twitch_oauth_uri,
            twitch::is_authenticated,
            twitch::logout,
            twitch::get_redeems_list,
            twitch::refresh_redeems_list,
            // Item manipulation commands
            items::get_item_by_id,
            items::get_items,
            items::create_item,
            items::update_item,
            items::update_item_orderings,
            items::delete_item,
            items::append_item_impact_sounds,
            // Sound commands
            sounds::get_sounds,
            sounds::get_sound_by_id,
            sounds::create_sound,
            sounds::update_sound,
            sounds::delete_sound,
            sounds::update_sound_orderings,
            // Command commands
            commands::get_commands,
            commands::get_command_by_id,
            commands::create_command,
            commands::update_command,
            commands::delete_command,
            commands::get_command_logs,
            commands::delete_command_logs,
            commands::update_command_orderings,
            commands::get_command_executions,
            commands::delete_command_executions,
            commands::export_commands,
            commands::import_commands,
            // Event commands
            events::get_events,
            events::get_event_by_id,
            events::create_event,
            events::update_event,
            events::delete_event,
            events::test_event_by_id,
            events::update_event_orderings,
            events::get_event_executions,
            events::delete_event_executions,
            events::get_event_logs,
            events::delete_event_logs,
            events::export_events,
            events::import_events,
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

    let http_port = block_on(AppDataModel::get_http_port(&db))
        .unwrap_or(database::entity::app_data::default_http_port());

    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (overlay_tx, overlay_rx) = create_overlay_channel();

    let twitch = Twitch::new(event_tx.clone());
    let overlay_data = OverlayDataStore::new(handle.clone());

    let script_handle = create_script_executor(
        app_data_path.join("modules"),
        ScriptRuntimeData {
            db: db.clone(),
            overlay_sender: overlay_tx.clone(),
            twitch: twitch.clone(),
        },
    );

    // Create background event scheduler
    let scheduler_handle = create_scheduler(event_tx);

    let storage = Storage::new_fs(handle)?;

    // Queue the scheduler events
    spawn({
        let db = db.clone();
        let scheduler_handle = scheduler_handle.clone();
        async move {
            update_scheduler_events(&db, &scheduler_handle).await;
        }
    });

    // Run background cleanup
    spawn(clean_old_data(db.clone()));

    // Provide overlay data store
    app.manage(overlay_data.clone());

    // Provide access to the scheduler
    app.manage(scheduler_handle);

    // Provide access to twitch manager and event sender
    app.manage(overlay_tx.clone());
    app.manage(twitch.clone());

    // Provide access to script running and
    app.manage(script_handle.clone());

    // Provide database access
    app.manage(db.clone());

    app.manage(storage.clone());
    app.manage(ServerPort(http_port));

    // Attempt to authenticate with twitch using the saved token
    _ = spawn({
        let twitch = twitch.clone();
        let db = db.clone();

        async move { twitch.attempt_auth_stored(db).await }
    });

    // Handle events triggered by twitch
    _ = spawn(process_events(
        db.clone(),
        twitch.clone(),
        script_handle,
        overlay_tx.clone(),
        handle.clone(),
        event_rx,
    ));

    match tauri::async_runtime::block_on(create_http_socket(http_port)) {
        Ok(http_socket) => {
            // Spawn HTTP server
            _ = spawn(http::start_http_server(
                http_socket,
                HttpExtensions {
                    db,
                    overlay_tx,
                    overlay_rx,
                    app_handle: handle.clone(),
                    twitch,
                    overlay_data,
                    storage,
                },
            ));
        }
        Err(cause) => {
            error!("failed to bind http server socket: {cause:?}");

            // Show error dialog about the failed port binding
            rfd::MessageDialog::new()
                .set_title("Failed to start")
                .set_description(format!(
                    "The port {} required to run VTFTK is currently in use, please change your port in settings 
                    or most features of VTFTK will be non-functional",
                    http_port
                ))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
    };

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
        let db = app.state::<DbPool>();
        let main_config = block_on(AppDataModel::get_main_config(db.inner()));
        let minimize_to_tray = main_config.is_ok_and(|value| value.minimize_to_tray);

        if code.is_none() && minimize_to_tray {
            api.prevent_exit();
        }
    }
}
