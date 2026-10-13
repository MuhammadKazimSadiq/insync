use serde_yaml::Value;

pub struct ExtractedMetadata {
    pub frontmatter: Value,
    pub body: String,
}

pub fn extract_frontmatter(content: &str) -> Option<ExtractedMetadata> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_start = &trimmed[3..];
    let first_newline = after_start.find('\n')?;
    let rest = &after_start[first_newline + 1..];

    // Find closing ---
    let mut end_index = None;
    let mut current_offset = 0;

    for line in rest.lines() {
        if line.trim() == "---" {
            end_index = Some(current_offset);
            break;
        }
        current_offset += line.len() + 1; // +1 for newline
    }

    let closing_pos = end_index?;
    let yaml_str = &rest[..closing_pos];
    let body = rest[closing_pos + 4..].to_string(); // Skip ---\n approx

    let frontmatter: Value = serde_yaml::from_str(yaml_str).ok()?;

    Some(ExtractedMetadata { frontmatter, body })
}
