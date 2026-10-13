use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::engine::PublisherEngine;
use publisher_core::manifest::load_manifest;
use publisher_core::plan::PlanAction;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_manifest_ledger_and_change_detection() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_dir = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_dir).expect("create Courses folder");

    let note_path = courses_dir.join("Note1.md");
    fs::write(&note_path, "---\nstatus: completed\n---\nInitial Content").expect("write note");

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "manifest-profile".to_string(),
            name: "Manifest Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let engine = PublisherEngine::new();

    // --- PASS 1: Initial Creation ---
    let plan1 = engine.preview(&config, "manifest-profile").expect("preview 1");
    assert_eq!(plan1.actions.len(), 1);
    match &plan1.actions[0] {
        PlanAction::Create { relative_path } => {
            assert_eq!(relative_path.to_str().unwrap(), "Courses/Note1.md");
        }
        _ => panic!("Expected Create action on pass 1"),
    }

    let report1 = engine.apply(plan1).expect("apply 1");
    assert_eq!(report1.created_count, 1);
    assert_eq!(report1.updated_count, 0);
    assert_eq!(report1.skipped_count, 0);

    // Verify manifest was written
    let manifest = load_manifest(dest_dir.path()).expect("load manifest");
    assert_eq!(manifest.profile_id, "manifest-profile");
    let record = manifest.files.get("Courses/Note1.md").expect("record in manifest");
    assert!(!record.source_hash.is_empty());
    assert_eq!(record.source_hash, record.destination_hash);

    // --- PASS 2: No Changes (Idempotency) ---
    let plan2 = engine.preview(&config, "manifest-profile").expect("preview 2");
    assert_eq!(plan2.actions.len(), 1);
    match &plan2.actions[0] {
        PlanAction::Skip { relative_path } => {
            assert_eq!(relative_path.to_str().unwrap(), "Courses/Note1.md");
        }
        _ => panic!("Expected Skip action on pass 2, got {:?}", plan2.actions[0]),
    }

    let report2 = engine.apply(plan2).expect("apply 2");
    assert_eq!(report2.created_count, 0);
    assert_eq!(report2.updated_count, 0);
    assert_eq!(report2.skipped_count, 1);

    // --- PASS 3: Modification (Update) ---
    fs::write(&note_path, "---\nstatus: completed\n---\nUpdated Content").expect("update note");

    let plan3 = engine.preview(&config, "manifest-profile").expect("preview 3");
    assert_eq!(plan3.actions.len(), 1);
    match &plan3.actions[0] {
        PlanAction::Update { relative_path } => {
            assert_eq!(relative_path.to_str().unwrap(), "Courses/Note1.md");
        }
        _ => panic!("Expected Update action on pass 3, got {:?}", plan3.actions[0]),
    }

    let report3 = engine.apply(plan3).expect("apply 3");
    assert_eq!(report3.created_count, 0);
    assert_eq!(report3.updated_count, 1);
    assert_eq!(report3.skipped_count, 0);

    // Verify destination was updated and manifest updated
    let dest_content = fs::read_to_string(dest_dir.path().join("Courses").join("Note1.md")).unwrap();
    assert!(dest_content.contains("Updated Content"));

    let manifest_updated = load_manifest(dest_dir.path()).expect("load updated manifest");
    let record_updated = manifest_updated.files.get("Courses/Note1.md").unwrap();
    assert_ne!(record.source_hash, record_updated.source_hash);
}
