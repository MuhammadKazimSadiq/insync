#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;

use app_state::AppState;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let default_config_path = PathBuf::from(home)
        .join(".config")
        .join("vault-publisher")
        .join("config.json");

    let state = AppState::new(default_config_path);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::preview_profile,
            commands::apply_profile,
            commands::reconcile_profile,
            commands::get_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
