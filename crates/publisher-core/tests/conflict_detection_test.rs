use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::engine::PublisherEngine;
use publisher_core::plan::PlanAction;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_destination_conflict_detection_and_preservation() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_src).unwrap();

    let note_a_src = courses_src.join("NoteA.md");
    let note_b_src = courses_src.join("NoteB.md");

    fs::write(&note_a_src, "---\nstatus: completed\n---\nNote A original").unwrap();
    fs::write(&note_b_src, "---\nstatus: completed\n---\nNote B original").unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "conflict-profile".to_string(),
            auto_sync: false,
            name: "Conflict Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let engine = PublisherEngine::new();

    // 1. Initial Publish
    let plan1 = engine.preview(&config, "conflict-profile").unwrap();
    assert_eq!(plan1.actions.len(), 2);
    let report1 = engine.apply(plan1).unwrap();
    assert_eq!(report1.created_count, 2);

    // 2. Collaborator modifies NoteA.md directly in destination
    let note_a_dst = dest_dir.path().join("Courses").join("NoteA.md");
    let collab_text = "COLLABORATOR EXTERNAL EDIT - DO NOT OVERWRITE";
    fs::write(&note_a_dst, collab_text).unwrap();

    // Author modifies NoteA.md and NoteB.md in source
    fs::write(&note_a_src, "---\nstatus: completed\n---\nNote A author edit").unwrap();
    fs::write(&note_b_src, "---\nstatus: completed\n---\nNote B author edit").unwrap();

    // 3. Preview should detect Conflict on NoteA.md
    let plan2 = engine.preview(&config, "conflict-profile").unwrap();

    let conflict_action = plan2.actions.iter().find(|a| {
        matches!(a, PlanAction::Conflict { relative_path, .. } if relative_path.to_str().unwrap() == "Courses/NoteA.md")
    });
    assert!(conflict_action.is_some(), "NoteA.md must be flagged as Conflict");

    let update_b = plan2.actions.iter().find(|a| {
        matches!(a, PlanAction::Update { relative_path } if relative_path.to_str().unwrap() == "Courses/NoteB.md")
    });
    assert!(update_b.is_some(), "NoteB.md must be scheduled for normal Update");

    // 4. Apply
    let report2 = engine.apply(plan2).unwrap();
    assert_eq!(report2.conflict_count, 1);
    assert_eq!(report2.updated_count, 1);

    // 5. Verification: NoteA destination content was preserved, not overwritten!
    let note_a_dst_after = fs::read_to_string(&note_a_dst).unwrap();
    assert_eq!(note_a_dst_after, collab_text, "Conflicting destination note must NOT be overwritten");

    // NoteB was updated normally
    let note_b_dst = dest_dir.path().join("Courses").join("NoteB.md");
    let note_b_dst_after = fs::read_to_string(&note_b_dst).unwrap();
    assert!(note_b_dst_after.contains("Note B author edit"));
}
