use crate::config::PublisherConfig;
use crate::error::PublisherError;
use crate::metadata::extract_frontmatter;
use crate::plan::{Plan, PlanAction, RunReport};
use std::fs;
use walkdir::WalkDir;

pub struct PublisherEngine;

impl PublisherEngine {
    pub fn new() -> Self {
        Self
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

                    if let Ok(content) = fs::read_to_string(path) {
                        if let Some(meta) = extract_frontmatter(&content) {
                            if let Some(status_val) = meta.frontmatter.get("status") {
                                if status_val.as_str() == Some("completed") {
                                    actions.push(PlanAction::Create { relative_path });
                                }
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
