use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::daemon::Daemon;
use publisher_core::error::PublisherError;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_daemon_single_instance_lock_and_timer() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_src).unwrap();

    let note_path = courses_src.join("Note.md");
    fs::write(&note_path, "---\nstatus: completed\n---\nDaemon Note").unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "daemon-profile".to_string(),
            name: "Daemon Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let tick_count = Arc::new(AtomicUsize::new(0));
    let tick_clone = Arc::clone(&tick_count);

    // 1. Start primary daemon with a 100ms periodic timer and 50ms quiet debouncer
    let mut daemon1 = Daemon::start(
        config.clone(),
        "daemon-profile".to_string(),
        Duration::from_millis(50),
        Duration::from_millis(100),
        move |_report| {
            tick_clone.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("daemon1 should acquire lock and start");

    // 2. Attempt to start a second daemon on the exact same profile -> MUST FAIL with LockError
    let daemon2_res = Daemon::start(
        config.clone(),
        "daemon-profile".to_string(),
        Duration::from_millis(50),
        Duration::from_millis(100),
        |_report| {},
    );

    assert!(
        matches!(daemon2_res, Err(PublisherError::LockError(_))),
        "Second daemon on same profile must fail with LockError"
    );

    // 3. Wait for periodic timer to trigger at least once
    thread::sleep(Duration::from_millis(250));
    assert!(
        tick_count.load(Ordering::SeqCst) >= 1,
        "Periodic reconciliation timer should have fired"
    );

    // 4. Stop daemon1
    daemon1.stop();

    // 5. Now starting daemon3 on the same profile should succeed because lock was released
    let daemon3_res = Daemon::start(
        config.clone(),
        "daemon-profile".to_string(),
        Duration::from_millis(50),
        Duration::from_millis(100),
        |_report| {},
    );
    assert!(
        daemon3_res.is_ok(),
        "Daemon should acquire lock after previous instance stopped"
    );
}
