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
    let insync_path = PathBuf::from(&home)
        .join(".config")
        .join("insync")
        .join("config.json");
    let legacy_path = PathBuf::from(&home)
        .join(".config")
        .join("vault-publisher")
        .join("config.json");

    let default_config_path = if legacy_path.exists() && !insync_path.exists() {
        legacy_path
    } else {
        insync_path
    };

    let state = AppState::new(default_config_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use objc2::{AllocAnyThread, MainThreadMarker};
                use objc2_app_kit::{NSApplication, NSImage};
                use objc2_foundation::NSData;

                let mtm = unsafe { MainThreadMarker::new_unchecked() };
                let ns_app = NSApplication::sharedApplication(mtm);
                let icon_bytes = include_bytes!("../icons/icon.png");
                let data = NSData::with_bytes(icon_bytes);
                let app_icon = NSImage::initWithData(NSImage::alloc(), &data);
                if let Some(ref icon) = app_icon {
                    unsafe { ns_app.setApplicationIconImage(Some(icon)) };
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            let open_item = MenuItem::with_id(app, "open", "Open InSync", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("InSync")
                .show_menu_on_left_click(true)
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
            commands::select_folder,
            commands::export_config_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
