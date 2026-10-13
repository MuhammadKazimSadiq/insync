use publisher_core::config::{ProfileConfig, PublisherConfig};
use publisher_core::engine::PublisherEngine;
use publisher_core::plan::PlanAction;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_minimal_publisher_tracer_bullet() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_dir = source_dir.path().join("Courses");
    fs::create_dir_all(&courses_dir).expect("create Courses folder");

    let lesson_content = "---\nstatus: completed\n---\n# Lesson 1\nContent of lesson 1";
    let draft_content = "---\nstatus: draft\n---\n# Draft\nContent of draft";
    let unmarked_content = "# No Frontmatter\nJust notes";

    fs::write(courses_dir.join("Lesson 1.md"), lesson_content).expect("write lesson 1");
    fs::write(courses_dir.join("Draft.md"), draft_content).expect("write draft");
    fs::write(courses_dir.join("Unmarked.md"), unmarked_content).expect("write unmarked");

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "test-profile".to_string(),
            name: "Test Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };

    let engine = PublisherEngine::new();

    // 1. Preview
    let plan = engine.preview(&config, "test-profile").expect("preview should succeed");
    assert_eq!(plan.actions.len(), 1, "only 1 note matches status: completed");
    match &plan.actions[0] {
        PlanAction::Create { relative_path } => {
            assert_eq!(relative_path.to_str().unwrap(), "Courses/Lesson 1.md");
        }
        _ => panic!("expected Create action"),
    }

    // 2. Apply
    let report = engine.apply(plan).expect("apply should succeed");
    assert_eq!(report.created_count, 1);

    // Verify destination files
    let dest_lesson = dest_dir.path().join("Courses").join("Lesson 1.md");
    assert!(dest_lesson.exists(), "Lesson 1.md must be copied to destination");
    assert_eq!(
        fs::read_to_string(&dest_lesson).expect("read dest file"),
        lesson_content
    );

    assert!(
        !dest_dir.path().join("Courses").join("Draft.md").exists(),
        "Draft.md must not be copied"
    );
    assert!(
        !dest_dir.path().join("Courses").join("Unmarked.md").exists(),
        "Unmarked.md must not be copied"
    );
}
