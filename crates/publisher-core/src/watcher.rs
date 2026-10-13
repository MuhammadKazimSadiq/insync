use crate::config::PublisherConfig;
use crate::engine::PublisherEngine;
use crate::plan::RunReport;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct DebounceEngine {
    pub quiet_period: Duration,
}

impl DebounceEngine {
    pub fn new(quiet_period: Duration) -> Self {
        Self { quiet_period }
    }

    /// Process a stream of raw event notifications, debouncing them into batched runs.
    pub fn run_loop<F>(
        &self,
        rx: Receiver<()>,
        stop_signal: Arc<AtomicBool>,
        mut on_trigger: F,
    ) where
        F: FnMut() -> (),
    {
        let mut last_event: Option<Instant> = None;

        while !stop_signal.load(Ordering::Relaxed) {
            // Check if new event arrived without blocking indefinitely
            if let Ok(()) = rx.recv_timeout(Duration::from_millis(20)) {
                last_event = Some(Instant::now());
            }

            if let Some(t) = last_event {
                if t.elapsed() >= self.quiet_period {
                    on_trigger();
                    last_event = None;
                }
            }
        }
    }
}

pub struct ScopeWatcher {
    _watcher: RecommendedWatcher,
    pub stop_signal: Arc<AtomicBool>,
    pub handle: Option<thread::JoinHandle<()>>,
}

impl ScopeWatcher {
    pub fn start<F>(
        config: PublisherConfig,
        profile_id: String,
        quiet_period: Duration,
        mut on_run_complete: F,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: FnMut(RunReport) + Send + 'static,
    {
        let profile = config
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| format!("Profile not found: {}", profile_id))?;

        let (tx, rx) = channel();
        let tx_notify = tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if !matches!(event.kind, EventKind::Access(_)) {
                        // Ignore internal staging files and hidden metadata
                        let is_noise = event.paths.iter().any(|p| {
                            p.components().any(|c| {
                                let name = c.as_os_str().to_string_lossy();
                                name.contains(".tmp.vault-publisher")
                                    || name == ".obsidian"
                                    || name == ".vault-publisher"
                            })
                        });
                        if !is_noise {
                            let _ = tx_notify.send(());
                        }
                    }
                }
            },
            Config::default(),
        )?;

        for scope in &profile.scopes {
            let scope_path = profile.source_vault.join(scope);
            if scope_path.exists() {
                let canon = scope_path.canonicalize().unwrap_or(scope_path);
                watcher.watch(&canon, RecursiveMode::Recursive)?;
            }
        }

        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_signal);

        let handle = thread::spawn(move || {
            let debouncer = DebounceEngine::new(quiet_period);
            let engine = PublisherEngine::new();

            debouncer.run_loop(rx, stop_clone, || {
                if let Ok(report) = engine.reconcile(&config, &profile_id) {
                    on_run_complete(report);
                }
            });
        });

        Ok(Self {
            _watcher: watcher,
            stop_signal,
            handle: Some(handle),
        })
    }

    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for ScopeWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}
