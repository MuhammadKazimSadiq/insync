use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::watcher::{DebounceEngine, ScopeWatcher};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_debounce_engine_batches_events() {
    let (tx, rx) = channel();
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_clone = Arc::clone(&stop_signal);

    let run_count = Arc::new(AtomicUsize::new(0));
    let run_count_clone = Arc::clone(&run_count);

    let engine = DebounceEngine::new(Duration::from_millis(80));

    let handle = thread::spawn(move || {
        engine.run_loop(rx, stop_clone, move || {
            run_count_clone.fetch_add(1, Ordering::SeqCst);
        });
    });

    // Send 5 rapid events within 30ms
    for _ in 0..5 {
        tx.send(()).unwrap();
        thread::sleep(Duration::from_millis(5));
    }

    // Wait 150ms for quiet window to trigger
    thread::sleep(Duration::from_millis(150));

    // Send another single event after quiet window
    tx.send(()).unwrap();
    thread::sleep(Duration::from_millis(150));

    stop_signal.store(true, Ordering::Relaxed);
    let _ = handle.join();

    assert_eq!(
        run_count.load(Ordering::SeqCst),
        2,
        "Expected exactly 2 debounced runs (1 for burst, 1 for second event)"
    );
}

#[test]
fn test_debounced_watcher_filesystem_integration() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_src).unwrap();

    let note_path = courses_src.join("LiveNote.md");
    fs::write(&note_path, "---\nstatus: completed\n---\nInitial").unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "watch-profile".to_string(),
            name: "Watch Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let run_count = Arc::new(AtomicUsize::new(0));
    let run_count_clone = Arc::clone(&run_count);

    let mut watcher = ScopeWatcher::start(
        config,
        "watch-profile".to_string(),
        Duration::from_millis(150),
        move |_report| {
            run_count_clone.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("start watcher");

    // Wait for FSEvents registration
    thread::sleep(Duration::from_millis(100));

    // Rapid edits
    for i in 1..=3 {
        fs::write(
            &note_path,
            format!("---\nstatus: completed\n---\nLive Edit {}", i),
        )
        .unwrap();
        thread::sleep(Duration::from_millis(15));
    }

    // Wait for debouncing quiet window and execution
    thread::sleep(Duration::from_millis(600));

    watcher.stop();

    let runs = run_count.load(Ordering::SeqCst);
    assert!(
        runs >= 1,
        "Watcher must have triggered at least 1 publication run, got {}",
        runs
    );

    // Verify destination was created with final content
    let dest_file = dest_dir.path().join("Courses").join("LiveNote.md");
    assert!(dest_file.exists());
    let dest_content = fs::read_to_string(dest_file).unwrap();
    assert!(dest_content.contains("Live Edit 3"));
}
