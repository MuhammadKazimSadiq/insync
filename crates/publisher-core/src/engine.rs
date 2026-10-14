use crate::attachments::{
    extract_attachment_references, resolve_attachment_path, AttachmentMode,
};
use crate::config::PublisherConfig;
use crate::error::PublisherError;
use crate::manifest::{
    compute_file_sha256, load_manifest, save_manifest, FileRecord, Manifest,
};
use crate::metadata::extract_frontmatter;
use crate::plan::{Plan, PlanAction, RunReport};
use crate::rules::evaluate_rule_group;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const ARCHIVE_DIR: &str = ".vault-publisher-archive";

pub struct PublisherEngine;

impl PublisherEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn reconcile(
        &self,
        config: &PublisherConfig,
        profile_id: &str,
    ) -> Result<RunReport, PublisherError> {
        let plan = self.preview(config, profile_id)?;
        self.apply(plan)
    }

    fn validate_profile_paths(&self, src: &Path, dst: &Path) -> Result<(), PublisherError> {
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
                let suffix = &exclude[1..];
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

        let existing_manifest = load_manifest(&profile.destination_vault).ok();
        let mut actions = Vec::new();
        let mut matched_paths = HashSet::new();
        let mut scheduled_attachment_paths = HashSet::new();

        let effective_scopes: Vec<String> = if profile.scopes.is_empty() {
            vec!["".to_string()]
        } else {
            profile.scopes.clone()
        };

        let att_mode = profile
            .attachments
            .as_ref()
            .map(|a| &a.mode)
            .unwrap_or(&AttachmentMode::Explicit);

        for scope in &effective_scopes {
            let scope_dir = if scope.is_empty() {
                profile.source_vault.clone()
            } else {
                profile.source_vault.join(scope)
            };

            if !scope_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&scope_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let relative_path = path
                    .strip_prefix(&profile.source_vault)
                    .map_err(|_| PublisherError::InvalidPath(path.display().to_string()))?
                    .to_path_buf();

                if self.is_excluded(&relative_path, &profile.excludes) {
                    continue;
                }

                let is_markdown = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
                    .unwrap_or(false);

                // Filter by attachment/file mode:
                if *att_mode == AttachmentMode::MarkdownOnly && !is_markdown {
                    continue;
                }
                if *att_mode == AttachmentMode::Referenced && !is_markdown {
                    // Non-markdown files are only synced when referenced by a matched markdown note
                    continue;
                }

                // Check rule matching:
                let (is_match, note_content) = if is_markdown {
                    let content = fs::read_to_string(path).ok();
                    let matches_rules = if let Some(rules) = &profile.rules {
                        if let Some(ref c) = content {
                            if let Some(meta) = extract_frontmatter(c) {
                                evaluate_rule_group(rules, &meta.frontmatter)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        // No rules active: all files in scope match
                        true
                    };
                    (matches_rules, content)
                } else {
                    // Non-markdown file: matches if no rules active
                    let matches_rules = profile.rules.is_none();
                    (matches_rules, None)
                };

                if is_match {
                    let rel_str = relative_path.to_string_lossy().to_string();
                    if matched_paths.insert(rel_str.clone()) {
                        let src_hash = compute_file_sha256(path).unwrap_or_default();
                        let dest_file = profile.destination_vault.join(&relative_path);

                        if let Some(ref manifest) = existing_manifest {
                            if let Some(record) = manifest.files.get(&rel_str) {
                                if dest_file.exists() {
                                    let current_dst_hash =
                                        compute_file_sha256(&dest_file).unwrap_or_default();
                                    if current_dst_hash != record.destination_hash {
                                        actions.push(PlanAction::Conflict {
                                            relative_path: relative_path.clone(),
                                            reason: "Destination file modified externally".to_string(),
                                        });
                                        continue;
                                    }
                                }

                                if record.source_hash == src_hash && dest_file.exists() {
                                    actions.push(PlanAction::Skip {
                                        relative_path: relative_path.clone(),
                                    });
                                } else {
                                    actions.push(PlanAction::Update {
                                        relative_path: relative_path.clone(),
                                    });
                                }
                            } else if dest_file.exists() {
                                actions.push(PlanAction::Update {
                                    relative_path: relative_path.clone(),
                                });
                            } else {
                                actions.push(PlanAction::Create {
                                    relative_path: relative_path.clone(),
                                });
                            }
                        } else if dest_file.exists() {
                            actions.push(PlanAction::Update {
                                relative_path: relative_path.clone(),
                            });
                        } else {
                            actions.push(PlanAction::Create {
                                relative_path: relative_path.clone(),
                            });
                        }

                        // If markdown and in Referenced mode, resolve attachment links
                        if let Some(ref content) = note_content {
                            if *att_mode == AttachmentMode::Referenced {
                                let att_refs = extract_attachment_references(content);
                                for att_ref in att_refs {
                                    if let Some(att_rel) = resolve_attachment_path(
                                        &profile.source_vault,
                                        &relative_path,
                                        &att_ref,
                                    ) {
                                        let is_scoped = profile.scopes.is_empty()
                                            || profile.scopes.iter().any(|s| att_rel.starts_with(s));

                                        if is_scoped && !self.is_excluded(&att_rel, &profile.excludes) {
                                            let att_rel_str = att_rel.to_string_lossy().to_string();
                                            if scheduled_attachment_paths.insert(att_rel_str.clone())
                                                && matched_paths.insert(att_rel_str.clone())
                                            {
                                                let att_src = profile.source_vault.join(&att_rel);
                                                let att_dst = profile.destination_vault.join(&att_rel);
                                                let att_src_hash =
                                                    compute_file_sha256(&att_src).unwrap_or_default();

                                                if let Some(ref manifest) = existing_manifest {
                                                    if let Some(record) = manifest.files.get(&att_rel_str) {
                                                        if att_dst.exists()
                                                            && record.source_hash == att_src_hash
                                                        {
                                                            actions.push(PlanAction::Skip {
                                                                relative_path: att_rel,
                                                            });
                                                            continue;
                                                        }
                                                    }
                                                }

                                                if att_dst.exists() {
                                                    actions.push(PlanAction::Update {
                                                        relative_path: att_rel,
                                                    });
                                                } else {
                                                    actions.push(PlanAction::Create {
                                                        relative_path: att_rel,
                                                    });
                                                }
                                            }
                                        }
                                    } else {
                                        actions.push(PlanAction::Warn {
                                            relative_path: None,
                                            reason: format!(
                                                "Referenced attachment '{}' in '{}' not found",
                                                att_ref, rel_str
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for previously managed files that are no longer matched or deleted
        if let Some(ref manifest) = existing_manifest {
            for managed_path in manifest.files.keys() {
                if !matched_paths.contains(managed_path) {
                    actions.push(PlanAction::Archive {
                        relative_path: PathBuf::from(managed_path),
                    });
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

        let mut manifest = load_manifest(&plan.destination_vault)
            .unwrap_or_else(|_| Manifest::new(&plan.profile_id));
        manifest.profile_id = plan.profile_id.clone();

        for action in plan.actions {
            match action {
                PlanAction::Create { relative_path } => {
                    let src = plan.source_vault.join(&relative_path);
                    let dst = plan.destination_vault.join(&relative_path);

                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let tmp_dst = dst.with_extension("tmp.vault-publisher");
                    fs::copy(&src, &tmp_dst)?;
                    fs::rename(&tmp_dst, &dst)?;

                    let src_hash = compute_file_sha256(&src)?;
                    let dst_hash = compute_file_sha256(&dst)?;
                    let rel_str = relative_path.to_string_lossy().to_string();

                    manifest.files.insert(
                        rel_str.clone(),
                        FileRecord {
                            relative_path: rel_str,
                            source_hash: src_hash,
                            destination_hash: dst_hash,
                            last_published: "now".to_string(),
                        },
                    );

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

                    let src_hash = compute_file_sha256(&src)?;
                    let dst_hash = compute_file_sha256(&dst)?;
                    let rel_str = relative_path.to_string_lossy().to_string();

                    manifest.files.insert(
                        rel_str.clone(),
                        FileRecord {
                            relative_path: rel_str,
                            source_hash: src_hash,
                            destination_hash: dst_hash,
                            last_published: "now".to_string(),
                        },
                    );

                    report.updated_count += 1;
                }
                PlanAction::Archive { relative_path } => {
                    let dst = plan.destination_vault.join(&relative_path);
                    let rel_str = relative_path.to_string_lossy().to_string();

                    if dst.exists() {
                        let archive_target = plan
                            .destination_vault
                            .join(ARCHIVE_DIR)
                            .join(&relative_path);
                        if let Some(parent) = archive_target.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::rename(&dst, &archive_target)?;
                    }

                    manifest.files.remove(&rel_str);
                    report.archived_count += 1;
                }
                PlanAction::Conflict { .. } => {
                    report.conflict_count += 1;
                }
                PlanAction::Warn { .. } => {
                    report.warn_count += 1;
                }
                PlanAction::Skip { .. } => {
                    report.skipped_count += 1;
                }
            }
        }

        save_manifest(&plan.destination_vault, &manifest)?;

        Ok(report)
    }
}
