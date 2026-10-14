use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::engine::PublisherEngine;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_reconciliation_and_idempotency() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_src).unwrap();

    let a_src = courses_src.join("A.md");
    let b_src = courses_src.join("B.md");

    fs::write(&a_src, "---\nstatus: completed\n---\nNote A v1").unwrap();
    fs::write(&b_src, "---\nstatus: completed\n---\nNote B v1").unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "recon-profile".to_string(),
            auto_sync: false,
            name: "Reconciliation Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let engine = PublisherEngine::new();

    // 1. Initial run
    let initial_report = engine.reconcile(&config, "recon-profile").unwrap();
    assert_eq!(initial_report.created_count, 2);

    // 2. Introduce divergence:
    // - Update A.md
    fs::write(&a_src, "---\nstatus: completed\n---\nNote A v2").unwrap();
    // - Delete B.md from source
    fs::remove_file(&b_src).unwrap();
    // - Add C.md
    fs::write(courses_src.join("C.md"), "---\nstatus: completed\n---\nNote C v1").unwrap();
    // - Add unmanaged file to destination
    fs::write(dest_dir.path().join("Courses").join("Unmanaged.md"), "Unmanaged content").unwrap();

    // 3. Reconcile
    let report = engine.reconcile(&config, "recon-profile").unwrap();
    assert_eq!(report.created_count, 1, "C.md created");
    assert_eq!(report.updated_count, 1, "A.md updated");
    assert_eq!(report.archived_count, 1, "B.md archived");

    // Check files
    assert!(dest_dir.path().join("Courses").join("C.md").exists());
    assert!(dest_dir.path().join("Courses").join("Unmanaged.md").exists());
    assert!(!dest_dir.path().join("Courses").join("B.md").exists());

    // 4. Idempotency pass: Reconcile again on unchanged vaults
    let report_idem = engine.reconcile(&config, "recon-profile").unwrap();
    assert_eq!(report_idem.created_count, 0, "Idempotent: 0 created");
    assert_eq!(report_idem.updated_count, 0, "Idempotent: 0 updated");
    assert_eq!(report_idem.archived_count, 0, "Idempotent: 0 archived");
    assert_eq!(report_idem.skipped_count, 2, "Idempotent: A.md and C.md skipped");
}
