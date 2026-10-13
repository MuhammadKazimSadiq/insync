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
