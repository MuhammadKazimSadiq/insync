use publisher_core::config::PublisherConfig;
use publisher_core::engine::PublisherEngine;
use publisher_core::history::HistoryEntry;
use publisher_core::watcher::ScopeWatcher;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

pub struct AppState {
    pub config_path: PathBuf,
    pub engine: PublisherEngine,
    pub config: Mutex<PublisherConfig>,
    pub history: Mutex<Vec<HistoryEntry>>,
    pub watchers: Mutex<HashMap<String, ScopeWatcher>>,
}

impl AppState {
    pub fn new(config_path: PathBuf) -> Self {
        let config = if config_path.exists() {
            let data = fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| PublisherConfig {
                version: 1,
                profiles: vec![],
            })
        } else {
            PublisherConfig {
                version: 1,
                profiles: vec![],
            }
        };

        let state = Self {
            config_path,
            engine: PublisherEngine::new(),
            config: Mutex::new(config),
            history: Mutex::new(vec![]),
            watchers: Mutex::new(HashMap::new()),
        };
        state.update_watchers();
        state
    }

    pub fn update_watchers(&self) {
        let config = match self.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return,
        };

        let mut watchers = match self.watchers.lock() {
            Ok(w) => w,
            Err(_) => return,
        };

        let active_ids: Vec<String> = config
            .profiles
            .iter()
            .filter(|p| p.auto_sync)
            .map(|p| p.id.clone())
            .collect();

        watchers.retain(|id, watcher| {
            if !active_ids.contains(id) {
                watcher.stop();
                false
            } else {
                true
            }
        });

        for profile in &config.profiles {
            if profile.auto_sync && !watchers.contains_key(&profile.id) {
                let cfg_clone = config.clone();
                let pid = profile.id.clone();
                if let Ok(w) = ScopeWatcher::start(
                    cfg_clone,
                    pid.clone(),
                    Duration::from_secs(2),
                    move |_report| {
                        // Background sync complete
                    },
                ) {
                    watchers.insert(pid, w);
                }
            }
        }
    }

    pub fn save_config(&self) -> Result<(), String> {
        let cfg = self.config.lock().map_err(|e| e.to_string())?;
        if let Some(parent) = self.config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&*cfg).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn record_history(&self, entry: HistoryEntry) {
        if let Ok(mut h) = self.history.lock() {
            h.insert(0, entry);
            if h.len() > 100 {
                h.truncate(100);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use publisher_core::config::{ProfileConfig, RuleGroup, RulePredicate};
    use publisher_core::rules::Operator;
    use serde_json::json;

    #[test]
    fn test_app_state_save_and_reload_with_rules() {
        let dir = std::env::temp_dir().join(format!("insync-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&dir);
        let config_file = dir.join("config.json");

        let state = AppState::new(config_file.clone());

        let profile = ProfileConfig {
            id: "sync-test-1".to_string(),
            auto_sync: false,
            name: "Work Notes Sync".to_string(),
            source_vault: dir.join("source"),
            destination_vault: dir.join("dest"),
            scopes: vec!["Notes".to_string()],
            excludes: vec![".git".to_string()],
            rules: Some(RuleGroup::all(vec![RuleGroup::Predicate(RulePredicate {
                property: "status".to_string(),
                operator: Operator::Equals,
                value: Some(json!("completed")),
            })])),
            attachments: None,
        };

        {
            let mut cfg = state.config.lock().unwrap();
            cfg.profiles.push(profile.clone());
        }

        state.save_config().expect("save config");

        // Reload state from disk
        let reloaded_state = AppState::new(config_file);
        let reloaded_cfg = reloaded_state.config.lock().unwrap();

        assert_eq!(reloaded_cfg.profiles.len(), 1);
        assert_eq!(reloaded_cfg.profiles[0].id, "sync-test-1");
        assert_eq!(reloaded_cfg.profiles[0].name, "Work Notes Sync");
        assert!(reloaded_cfg.profiles[0].rules.is_some());
        match reloaded_cfg.profiles[0].rules.as_ref().unwrap() {
            RuleGroup::All { all } => {
                assert_eq!(all.len(), 1);
            }
            other => panic!("expected RuleGroup::All, got {:?}", other),
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
