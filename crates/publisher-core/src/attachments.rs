use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentMode {
    MarkdownOnly,
    Explicit,
    Referenced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentConfig {
    pub mode: AttachmentMode,
}

pub fn extract_attachment_references(content: &str) -> Vec<String> {
    let mut refs = Vec::new();

    // 1. Wikilinks: ![[filename.ext]]
    let mut cursor = 0;
    while let Some(start) = content[cursor..].find("![[") {
        let abs_start = cursor + start + 3;
        if let Some(end) = content[abs_start..].find("]]") {
            let ref_str = &content[abs_start..abs_start + end];
            // Split alias or pipe if present: ![[file.png|100]]
            let clean_ref = ref_str.split('|').next().unwrap_or(ref_str).trim();
            if !clean_ref.is_empty() {
                refs.push(clean_ref.to_string());
            }
            cursor = abs_start + end + 2;
        } else {
            break;
        }
    }

    // 2. Standard Markdown: ![alt](path/file.ext)
    let mut cursor = 0;
    while let Some(start) = content[cursor..].find("![") {
        let after_bracket = cursor + start + 2;
        if let Some(close_bracket) = content[after_bracket..].find("](") {
            let paren_start = after_bracket + close_bracket + 2;
            if let Some(close_paren) = content[paren_start..].find(')') {
                let link_target = &content[paren_start..paren_start + close_paren].trim();
                if !link_target.is_empty() && !link_target.starts_with("http://") && !link_target.starts_with("https://") {
                    refs.push(link_target.to_string());
                }
                cursor = paren_start + close_paren + 1;
            } else {
                break;
            }
        } else {
            cursor = after_bracket;
        }
    }

    refs
}

pub fn resolve_attachment_path(
    source_vault: &Path,
    note_relative_path: &Path,
    attachment_ref: &str,
) -> Option<PathBuf> {
    let note_parent = note_relative_path.parent().unwrap_or(Path::new(""));

    // Check sibling / subfolder relative to note
    let candidate1 = source_vault.join(note_parent).join(attachment_ref);
    if candidate1.exists() {
        return candidate1.strip_prefix(source_vault).ok().map(|p| p.to_path_buf());
    }

    // Check directly relative to source vault
    let candidate2 = source_vault.join(attachment_ref);
    if candidate2.exists() {
        return candidate2.strip_prefix(source_vault).ok().map(|p| p.to_path_buf());
    }

    None
}
