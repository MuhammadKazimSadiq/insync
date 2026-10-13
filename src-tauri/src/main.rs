#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;

use app_state::AppState;
use std::path::PathBuf;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let default_config_path = PathBuf::from(home)
        .join(".config")
        .join("vault-publisher")
        .join("config.json");

    let state = AppState::new(default_config_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            let open_item = MenuItem::with_id(app, "open", "Open Vault Publisher", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::preview_profile,
            commands::apply_profile,
            commands::reconcile_profile,
            commands::get_run_history,
            commands::get_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
