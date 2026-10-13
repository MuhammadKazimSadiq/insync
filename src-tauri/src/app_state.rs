use publisher_core::config::PublisherConfig;
use publisher_core::engine::PublisherEngine;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub config_path: PathBuf,
    pub engine: PublisherEngine,
    pub config: Mutex<PublisherConfig>,
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

        Self {
            config_path,
            engine: PublisherEngine::new(),
            config: Mutex::new(config),
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
}
