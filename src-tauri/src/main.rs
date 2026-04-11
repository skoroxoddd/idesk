#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("remote_desktop_app=debug,remote_desktop_core=debug")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::connection::get_session_id,
            commands::connection::connect_to_peer,
            commands::connection::disconnect,
            commands::connection::check_peer_online,
            commands::settings::set_quality,
            commands::settings::set_fps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
