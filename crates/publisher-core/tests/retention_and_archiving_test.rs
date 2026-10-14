use publisher_core::config::{ProfileConfig, PublisherConfig, RuleGroup, RulePredicate};
use publisher_core::engine::PublisherEngine;
use publisher_core::manifest::load_manifest;
use publisher_core::plan::PlanAction;
use publisher_core::rules::Operator;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_retention_and_safe_archiving() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_src).unwrap();

    let keep_src = courses_src.join("Keep.md");
    let drop_src = courses_src.join("Drop.md");

    fs::write(&keep_src, "---\nstatus: completed\n---\nKeep this").unwrap();
    fs::write(&drop_src, "---\nstatus: completed\n---\nDrop this later").unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "archive-profile".to_string(),
            auto_sync: false,
            name: "Archive Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: Some(RuleGroup::all(vec![RuleGroup::Predicate(RulePredicate {
                property: "status".to_string(),
                operator: Operator::Equals,
                value: Some(json!("completed")),
            })])),
            attachments: None,
        }],
    };

    let engine = PublisherEngine::new();

    // 1. Initial Publish
    let plan1 = engine.preview(&config, "archive-profile").unwrap();
    assert_eq!(plan1.actions.len(), 2);
    let report1 = engine.apply(plan1).unwrap();
    assert_eq!(report1.created_count, 2);

    // Add unmanaged collaborator file directly to destination
    let collab_file = dest_dir.path().join("Courses").join("CollaboratorNote.md");
    fs::write(&collab_file, "# Collaborator Work\nDo not delete!").unwrap();

    // 2. Change Drop.md to draft in source
    fs::write(&drop_src, "---\nstatus: draft\n---\nNow a draft").unwrap();

    // 3. Preview should detect Drop.md needs Archive
    let plan2 = engine.preview(&config, "archive-profile").unwrap();

    let archive_action = plan2
        .actions
        .iter()
        .find(|a| matches!(a, PlanAction::Archive { relative_path } if relative_path.to_str().unwrap() == "Courses/Drop.md"));
    assert!(
        archive_action.is_some(),
        "Plan must include Archive action for Drop.md"
    );

    // 4. Apply
    let report2 = engine.apply(plan2).unwrap();
    assert_eq!(report2.archived_count, 1);

    // Verify destination states:
    // - Drop.md no longer in destination Courses/
    assert!(
        !dest_dir.path().join("Courses").join("Drop.md").exists(),
        "Drop.md must be removed from active destination path"
    );

    // - Drop.md moved into archive directory
    let archive_dir = dest_dir.path().join(".vault-publisher-archive");
    assert!(archive_dir.exists(), "Archive directory must exist");

    // - Collaborator note untouched
    assert!(
        collab_file.exists(),
        "Unmanaged collaborator note must NEVER be deleted"
    );

    // - Keep note intact
    assert!(
        dest_dir.path().join("Courses").join("Keep.md").exists(),
        "Keep note must remain in destination"
    );

    // - Manifest updated: Drop.md removed from managed files
    let manifest = load_manifest(dest_dir.path()).unwrap();
    assert!(!manifest.files.contains_key("Courses/Drop.md"));
    assert!(manifest.files.contains_key("Courses/Keep.md"));
}
