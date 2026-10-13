use crate::config::PublisherConfig;
use crate::error::PublisherError;
use crate::metadata::extract_frontmatter;
use crate::plan::{Plan, PlanAction, RunReport};
use crate::rules::evaluate_rule_group;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct PublisherEngine;

impl PublisherEngine {
    pub fn new() -> Self {
        Self
    }

    fn validate_profile_paths(
        &self,
        src: &Path,
        dst: &Path,
    ) -> Result<(), PublisherError> {
        let src_canon = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
        let dst_canon = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());

        if src_canon == dst_canon {
            return Err(PublisherError::UnsafePath(
                "Source and destination vaults are identical".to_string(),
            ));
        }

        if dst_canon.starts_with(&src_canon) {
            return Err(PublisherError::UnsafePath(
                "Destination vault is nested inside source vault".to_string(),
            ));
        }

        Ok(())
    }

    fn is_excluded(&self, relative_path: &Path, excludes: &[String]) -> bool {
        let rel_str = relative_path.to_string_lossy();
        let filename = relative_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for component in relative_path.components() {
            let comp_str = component.as_os_str().to_string_lossy();
            if comp_str.starts_with('.') && comp_str != "." && comp_str != ".." {
                return true;
            }
            for exclude in excludes {
                if comp_str == *exclude {
                    return true;
                }
            }
        }

        for exclude in excludes {
            if exclude.starts_with("*.") {
                let suffix = &exclude[1..]; // e.g. ".draft.md"
                if filename.ends_with(suffix) || rel_str.ends_with(suffix) {
                    return true;
                }
            } else if rel_str.contains(exclude) {
                return true;
            }
        }

        false
    }

    pub fn preview(
        &self,
        config: &PublisherConfig,
        profile_id: &str,
    ) -> Result<Plan, PublisherError> {
        let profile = config
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| PublisherError::ProfileNotFound(profile_id.to_string()))?;

        self.validate_profile_paths(&profile.source_vault, &profile.destination_vault)?;

        let mut actions = Vec::new();

        for scope in &profile.scopes {
            let scope_dir = profile.source_vault.join(scope);
            if !scope_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&scope_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let relative_path = path
                        .strip_prefix(&profile.source_vault)
                        .map_err(|_| PublisherError::InvalidPath(path.display().to_string()))?
                        .to_path_buf();

                    if self.is_excluded(&relative_path, &profile.excludes) {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(path) {
                        if let Some(meta) = extract_frontmatter(&content) {
                            let is_match = if let Some(rules) = &profile.rules {
                                evaluate_rule_group(rules, &meta.frontmatter)
                            } else {
                                // Default rule: status equals completed
                                meta.frontmatter
                                    .get("status")
                                    .and_then(|v| v.as_str())
                                    == Some("completed")
                            };

                            if is_match {
                                actions.push(PlanAction::Create { relative_path });
                            }
                        }
                    }
                }
            }
        }

        Ok(Plan {
            profile_id: profile.id.clone(),
            source_vault: profile.source_vault.clone(),
            destination_vault: profile.destination_vault.clone(),
            actions,
        })
    }

    pub fn apply(&self, plan: Plan) -> Result<RunReport, PublisherError> {
        let mut report = RunReport {
            profile_id: plan.profile_id.clone(),
            ..Default::default()
        };

        for action in plan.actions {
            match action {
                PlanAction::Create { relative_path } => {
                    let src = plan.source_vault.join(&relative_path);
                    let dst = plan.destination_vault.join(&relative_path);

                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Temporary staging file for atomic replacement
                    let tmp_dst = dst.with_extension("tmp.vault-publisher");
                    fs::copy(&src, &tmp_dst)?;
                    fs::rename(&tmp_dst, &dst)?;

                    report.created_count += 1;
                }
                PlanAction::Update { relative_path } => {
                    let src = plan.source_vault.join(&relative_path);
                    let dst = plan.destination_vault.join(&relative_path);

                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let tmp_dst = dst.with_extension("tmp.vault-publisher");
                    fs::copy(&src, &tmp_dst)?;
                    fs::rename(&tmp_dst, &dst)?;

                    report.updated_count += 1;
                }
                PlanAction::Archive { .. } => {
                    report.archived_count += 1;
                }
                PlanAction::Skip { .. } => {
                    report.skipped_count += 1;
                }
            }
        }

        Ok(report)
    }
}
