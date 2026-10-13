use crate::plan::RunReport;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub profile_id: String,
    pub created: usize,
    pub updated: usize,
    pub archived: usize,
    pub skipped: usize,
    pub conflicts: usize,
    pub warnings: usize,
    pub duration_ms: u64,
}

impl HistoryEntry {
    pub fn from_report(report: &RunReport, duration_ms: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: format!("run-{}", now),
            timestamp: format!("{}", now),
            profile_id: report.profile_id.clone(),
            created: report.created_count,
            updated: report.updated_count,
            archived: report.archived_count,
            skipped: report.skipped_count,
            conflicts: report.conflict_count,
            warnings: report.warn_count,
            duration_ms,
        }
    }
}
