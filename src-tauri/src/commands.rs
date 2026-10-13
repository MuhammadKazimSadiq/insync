use crate::app_state::AppState;
use publisher_core::config::ProfileConfig;
use publisher_core::plan::{Plan, RunReport};
use serde_json::{json, Value};
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
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    let plan = state
        .engine
        .preview(&cfg, &profile_id)
        .map_err(|e| e.to_string())?;
    state.engine.apply(plan).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reconcile_profile(profile_id: String, state: State<AppState>) -> Result<RunReport, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    state
        .engine
        .reconcile(&cfg, &profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_diagnostics(state: State<AppState>) -> Result<Value, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(json!({
        "version": "0.1.0",
        "profilesCount": cfg.profiles.len(),
        "profiles": cfg.profiles.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
        "configPath": state.config_path.to_string_lossy(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH
    }))
}
