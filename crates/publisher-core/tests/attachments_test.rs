use publisher_core::config::{AttachmentConfig, AttachmentMode, ProfileConfig, PublisherConfig};
use publisher_core::engine::PublisherEngine;
use publisher_core::plan::PlanAction;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_attachment_policies_and_wikilink_resolution() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    let courses_src = source_dir.path().join("Courses");
    let assets_src = courses_src.join("assets");
    fs::create_dir_all(&assets_src).unwrap();

    let diagram_bytes = b"PNG_DIAGRAM_DATA";
    let photo_bytes = b"JPG_PHOTO_DATA";

    fs::write(courses_src.join("diagram.png"), diagram_bytes).unwrap();
    fs::write(assets_src.join("photo.jpg"), photo_bytes).unwrap();

    let note_content = r#"---
status: completed
---
# Lesson
Here is the diagram: ![[diagram.png]]
Here is the photo: ![Photo](assets/photo.jpg)
Here is a missing file: ![[missing.pdf]]
"#;
    fs::write(courses_src.join("Lesson.md"), note_content).unwrap();

    let config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "attachment-profile".to_string(),
            name: "Attachment Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: Some(AttachmentConfig {
                mode: AttachmentMode::Referenced,
            }),
        }],
    };

    let engine = PublisherEngine::new();

    // 1. Preview
    let plan = engine.preview(&config, "attachment-profile").unwrap();

    // Should contain:
    // Create: Courses/Lesson.md
    // Create: Courses/diagram.png
    // Create: Courses/assets/photo.jpg
    // Warn: missing.pdf
    let has_diagram = plan.actions.iter().any(|a| {
        matches!(a, PlanAction::Create { relative_path } if relative_path.to_str().unwrap() == "Courses/diagram.png")
    });
    assert!(has_diagram, "diagram.png must be scheduled for Create");

    let has_photo = plan.actions.iter().any(|a| {
        matches!(a, PlanAction::Create { relative_path } if relative_path.to_str().unwrap() == "Courses/assets/photo.jpg")
    });
    assert!(has_photo, "assets/photo.jpg must be scheduled for Create");

    let has_missing_warn = plan.actions.iter().any(|a| {
        matches!(a, PlanAction::Warn { reason, .. } if reason.contains("missing.pdf"))
    });
    assert!(has_missing_warn, "missing.pdf must generate a Warn action");

    // 2. Apply
    let report = engine.apply(plan).unwrap();
    assert_eq!(report.warn_count, 1, "Report must reflect 1 warning for missing attachment");
    assert_eq!(report.created_count, 3, "Lesson.md + diagram.png + photo.jpg = 3 created");

    // 3. Verify destination files exist
    let dst_diagram = dest_dir.path().join("Courses").join("diagram.png");
    let dst_photo = dest_dir.path().join("Courses").join("assets").join("photo.jpg");
    assert!(dst_diagram.exists(), "Destination diagram.png must exist");
    assert!(dst_photo.exists(), "Destination assets/photo.jpg must exist");
    assert_eq!(fs::read(&dst_diagram).unwrap(), diagram_bytes);
    assert_eq!(fs::read(&dst_photo).unwrap(), photo_bytes);
}
