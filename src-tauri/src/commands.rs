use crate::app_state::AppState;
use publisher_core::config::ProfileConfig;
use publisher_core::history::HistoryEntry;
use publisher_core::plan::{Plan, RunReport};
use serde_json::{json, Value};
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> Result<Vec<ProfileConfig>, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.profiles.clone())
}

#[tauri::command]
pub fn save_profile(profile: ProfileConfig, state: State<AppState>) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = cfg.profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile;
        } else {
            cfg.profiles.push(profile);
        }
    }
    state.save_config()
}

#[tauri::command]
pub fn delete_profile(profile_id: String, state: State<AppState>) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.profiles.retain(|p| p.id != profile_id);
    }
    state.save_config()
}

#[tauri::command]
pub fn preview_profile(profile_id: String, state: State<AppState>) -> Result<Plan, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    state
        .engine
        .preview(&cfg, &profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_profile(profile_id: String, state: State<AppState>) -> Result<RunReport, String> {
    let start = Instant::now();
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    let plan = state
        .engine
        .preview(&cfg, &profile_id)
        .map_err(|e| e.to_string())?;
    let report = state.engine.apply(plan).map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as u64;
    state.record_history(HistoryEntry::from_report(&report, duration));
    Ok(report)
}

#[tauri::command]
pub fn reconcile_profile(profile_id: String, state: State<AppState>) -> Result<RunReport, String> {
    let start = Instant::now();
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    let report = state
        .engine
        .reconcile(&cfg, &profile_id)
        .map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as u64;
    state.record_history(HistoryEntry::from_report(&report, duration));
    Ok(report)
}

#[tauri::command]
pub fn get_run_history(state: State<AppState>) -> Result<Vec<HistoryEntry>, String> {
    let h = state.history.lock().map_err(|e| e.to_string())?;
    Ok(h.clone())
}

#[tauri::command]
pub fn get_diagnostics(state: State<AppState>) -> Result<Value, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    let history_count = state.history.lock().map(|h| h.len()).unwrap_or(0);
    Ok(json!({
        "app": "InSync",
        "version": "0.1.0",
        "syncedFoldersCount": cfg.profiles.len(),
        "syncedFolders": cfg.profiles.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
        "historyEntriesCount": history_count,
        "configPath": state.config_path.to_string_lossy(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH
    }))
}

#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app.dialog().file().blocking_pick_folder();
    let res = path.map(|fp| match fp {
        tauri_plugin_dialog::FilePath::Path(p) => p.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(u) => u
            .to_file_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| u.to_string()),
    });
    Ok(res)
}

#[tauri::command]
pub async fn export_config_file(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let cfg_json = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&*cfg).map_err(|e| e.to_string())?
    };

    let path = app
        .dialog()
        .file()
        .set_file_name("insync-config.json")
        .add_filter("JSON Configuration", &["json"])
        .blocking_save_file();

    let target = path.map(|fp| match fp {
        tauri_plugin_dialog::FilePath::Path(p) => p.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(u) => u
            .to_file_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| u.to_string()),
    });

    if let Some(ref path_str) = target {
        std::fs::write(path_str, cfg_json).map_err(|e| format!("Failed to write file: {}", e))?;
    }
    Ok(target)
}
