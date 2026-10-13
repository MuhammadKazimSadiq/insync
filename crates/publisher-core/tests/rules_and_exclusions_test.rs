use publisher_core::config::{ProfileConfig, PublisherConfig, RuleGroup, RulePredicate};
use publisher_core::engine::PublisherEngine;
use publisher_core::error::PublisherError;
use publisher_core::rules::{evaluate_rule_group, Operator};
use serde_json::json;
use serde_yaml::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_rule_operators_and_nested_groups() {
    let note_meta_yaml = r#"
status: completed
share: true
visibility: private
tags:
  - rust
  - public
category: course
count: 42
"#;
    let yaml_val: Value = serde_yaml::from_str(note_meta_yaml).expect("parse yaml");

    // 1. Equals & strict type check: boolean true != "true"
    let equals_true_bool = RulePredicate {
        property: "share".to_string(),
        operator: Operator::Equals,
        value: Some(json!(true)),
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(equals_true_bool), &yaml_val));

    let equals_true_str = RulePredicate {
        property: "share".to_string(),
        operator: Operator::Equals,
        value: Some(json!("true")),
    };
    assert!(
        !evaluate_rule_group(&RuleGroup::Predicate(equals_true_str), &yaml_val),
        "Strict type check: bool true must not equal string 'true'"
    );

    // 2. notEquals
    let not_equals = RulePredicate {
        property: "status".to_string(),
        operator: Operator::NotEquals,
        value: Some(json!("archived")),
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(not_equals), &yaml_val));

    // 3. exists & notExists
    let exists = RulePredicate {
        property: "tags".to_string(),
        operator: Operator::Exists,
        value: None,
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(exists), &yaml_val));

    let not_exists = RulePredicate {
        property: "nonexistent".to_string(),
        operator: Operator::NotExists,
        value: None,
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(not_exists), &yaml_val));

    // 4. contains (array contains element)
    let contains_tag = RulePredicate {
        property: "tags".to_string(),
        operator: Operator::Contains,
        value: Some(json!("public")),
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(contains_tag), &yaml_val));

    let contains_missing_tag = RulePredicate {
        property: "tags".to_string(),
        operator: Operator::Contains,
        value: Some(json!("confidential")),
    };
    assert!(!evaluate_rule_group(&RuleGroup::Predicate(contains_missing_tag), &yaml_val));

    // 5. in (value in array)
    let in_array = RulePredicate {
        property: "category".to_string(),
        operator: Operator::In,
        value: Some(json!(["course", "guide"])),
    };
    assert!(evaluate_rule_group(&RuleGroup::Predicate(in_array), &yaml_val));

    // 6. Nested All + Any
    let nested_group = RuleGroup::All(vec![
        RuleGroup::Predicate(RulePredicate {
            property: "status".to_string(),
            operator: Operator::Equals,
            value: Some(json!("completed")),
        }),
        RuleGroup::Any(vec![
            RuleGroup::Predicate(RulePredicate {
                property: "visibility".to_string(),
                operator: Operator::Equals,
                value: Some(json!("public")),
            }),
            RuleGroup::Predicate(RulePredicate {
                property: "share".to_string(),
                operator: Operator::Equals,
                value: Some(json!(true)),
            }),
        ]),
    ]);
    assert!(evaluate_rule_group(&nested_group, &yaml_val));
}

#[test]
fn test_path_safety_and_exclusions() {
    let source_dir = tempdir().expect("create source temp dir");
    let dest_dir = tempdir().expect("create dest temp dir");

    // Safety: Source == Destination rejected
    let engine = PublisherEngine::new();
    let unsafe_same_config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "bad-profile".to_string(),
            name: "Bad Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: source_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };
    let err = engine.preview(&unsafe_same_config, "bad-profile").unwrap_err();
    match err {
        PublisherError::UnsafePath(msg) => {
            assert!(msg.contains("identical") || msg.contains("same"), "Expected path safety error: {}", msg);
        }
        other => panic!("expected UnsafePath error, got {:?}", other),
    }

    // Safety: Destination inside Source rejected
    let nested_dest = source_dir.path().join("nested_dest");
    fs::create_dir_all(&nested_dest).unwrap();
    let unsafe_nested_config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "bad-nested".to_string(),
            name: "Bad Nested".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: nested_dest,
            scopes: vec!["Courses".to_string()],
            excludes: vec![],
            rules: None,
            attachments: None,
        }],
    };
    let err_nested = engine.preview(&unsafe_nested_config, "bad-nested").unwrap_err();
    match err_nested {
        PublisherError::UnsafePath(msg) => {
            assert!(msg.contains("inside") || msg.contains("nested"), "Expected nested error: {}", msg);
        }
        other => panic!("expected UnsafePath error, got {:?}", other),
    }

    // Exclusions: .obsidian, _private, and glob patterns
    let courses_dir = source_dir.path().join("Courses");
    let obsidian_dir = courses_dir.join(".obsidian");
    let private_dir = courses_dir.join("_private");
    fs::create_dir_all(&obsidian_dir).unwrap();
    fs::create_dir_all(&private_dir).unwrap();

    let valid_yaml = "---\nstatus: completed\n---\n# Valid";
    fs::write(courses_dir.join("Valid.md"), valid_yaml).unwrap();
    fs::write(obsidian_dir.join("config.md"), valid_yaml).unwrap();
    fs::write(private_dir.join("Secret.md"), valid_yaml).unwrap();
    fs::write(courses_dir.join("Note.draft.md"), valid_yaml).unwrap();

    let safe_config = PublisherConfig {
        version: 1,
        profiles: vec![ProfileConfig {
            id: "safe-profile".to_string(),
            name: "Safe Profile".to_string(),
            source_vault: source_dir.path().to_path_buf(),
            destination_vault: dest_dir.path().to_path_buf(),
            scopes: vec!["Courses".to_string()],
            excludes: vec![
                ".obsidian".to_string(),
                "_private".to_string(),
                "*.draft.md".to_string(),
            ],
            rules: None,
            attachments: None,
        }],
    };

    let plan = engine.preview(&safe_config, "safe-profile").expect("preview should succeed");
    assert_eq!(plan.actions.len(), 1, "Only Valid.md should match, others excluded");
}
