pub use crate::attachments::{AttachmentConfig, AttachmentMode};
pub use crate::rules::{RuleGroup, RulePredicate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherConfig {
    pub version: u32,
    pub profiles: Vec<ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub source_vault: PathBuf,
    pub destination_vault: PathBuf,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub rules: Option<RuleGroup>,
    #[serde(default)]
    pub attachments: Option<AttachmentConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_profile_from_frontend_payload() {
        let json_str = r#"{
            "id": "shared-notes",
            "name": "Shared Documents",
            "sourceVault": "/Users/documents/Source",
            "destinationVault": "/Users/documents/Destination",
            "scopes": ["Documents", "Projects/Shared"],
            "excludes": [".git", "_private", "*.draft.md"],
            "rules": {
                "all": [
                    { "property": "status", "operator": "equals", "value": "completed" },
                    { "property": "share", "operator": "equals", "value": true }
                ]
            },
            "attachments": { "mode": "referenced" }
        }"#;

        let profile: ProfileConfig = serde_json::from_str(json_str).expect("deserialize profile");
        assert_eq!(profile.id, "shared-notes");
        assert_eq!(profile.name, "Shared Documents");
        assert_eq!(profile.source_vault, PathBuf::from("/Users/documents/Source"));
        assert_eq!(profile.destination_vault, PathBuf::from("/Users/documents/Destination"));
        assert_eq!(profile.scopes, vec!["Documents", "Projects/Shared"]);
        assert_eq!(profile.excludes, vec![".git", "_private", "*.draft.md"]);
        assert!(profile.rules.is_some());
        match profile.rules.unwrap() {
            RuleGroup::All { all } => {
                assert_eq!(all.len(), 2);
            }
            other => panic!("expected RuleGroup::All, got {:?}", other),
        }
        assert!(profile.attachments.is_some());
        assert_eq!(profile.attachments.unwrap().mode, AttachmentMode::Referenced);
    }

    #[test]
    fn test_deserialize_profile_with_null_rules() {
        let json_str = r#"{
            "id": "empty-rules",
            "name": "No Rules",
            "sourceVault": "/Users/documents/Source",
            "destinationVault": "/Users/documents/Destination",
            "scopes": ["Documents"],
            "excludes": [],
            "rules": null,
            "attachments": null
        }"#;

        let profile: ProfileConfig = serde_json::from_str(json_str).expect("deserialize profile with null rules");
        assert!(profile.rules.is_none());
        assert!(profile.attachments.is_none());
    }
}
