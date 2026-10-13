use crate::config::PublisherConfig;
use crate::engine::PublisherEngine;
use crate::error::PublisherError;
use crate::manifest::MANIFEST_DIR;
use crate::plan::RunReport;
use crate::watcher::ScopeWatcher;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ProfileLockGuard {
    lock_path: PathBuf,
}

impl ProfileLockGuard {
    pub fn acquire(dest_vault: &std::path::Path, profile_id: &str) -> Result<Self, PublisherError> {
        let dir = dest_vault.join(MANIFEST_DIR);
        fs::create_dir_all(&dir)?;

        let lock_path = dir.join(format!("{}.lock", profile_id));
        match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(mut file) => {
                let _ = writeln!(file, "PID: {}", std::process::id());
                Ok(Self { lock_path })
            }
            Err(_) => Err(PublisherError::LockError(format!(
                "Profile '{}' is already locked by another running agent",
                profile_id
            ))),
        }
    }
}

impl Drop for ProfileLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

pub struct Daemon {
    lock: Option<ProfileLockGuard>,
    mut_watcher: ScopeWatcher,
    stop_signal: Arc<AtomicBool>,
    timer_handle: Option<thread::JoinHandle<()>>,
}

impl Daemon {
    pub fn start<F>(
        config: PublisherConfig,
        profile_id: String,
        quiet_period: Duration,
        periodic_interval: Duration,
        on_report: F,
    ) -> Result<Self, PublisherError>
    where
        F: Fn(RunReport) + Send + Sync + 'static,
    {
        let profile = config
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| PublisherError::ProfileNotFound(profile_id.clone()))?;

        // 1. Acquire single-instance profile lock
        let lock = ProfileLockGuard::acquire(&profile.destination_vault, &profile_id)?;

        let on_report_arc = Arc::new(on_report);

        // 2. Start debounced filesystem watcher
        let on_watcher_report = Arc::clone(&on_report_arc);
        let watcher = ScopeWatcher::start(
            config.clone(),
            profile_id.clone(),
            quiet_period,
            move |report| {
                on_watcher_report(report);
            },
        )
        .map_err(|e| PublisherError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 3. Start periodic reconciliation timer
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_signal);
        let on_timer_report = Arc::clone(&on_report_arc);
        let config_timer = config;
        let profile_id_timer = profile_id;

        let timer_handle = thread::spawn(move || {
            let engine = PublisherEngine::new();
            while !stop_clone.load(Ordering::Relaxed) {
                thread::sleep(periodic_interval);
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(report) = engine.reconcile(&config_timer, &profile_id_timer) {
                    on_timer_report(report);
                }
            }
        });

        Ok(Self {
            lock: Some(lock),
            mut_watcher: watcher,
            stop_signal,
            timer_handle: Some(timer_handle),
        })
    }

    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        self.mut_watcher.stop();
        if let Some(h) = self.timer_handle.take() {
            let _ = h.join();
        }
        self.lock.take(); // Release lock explicitly
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        self.stop();
    }
}
