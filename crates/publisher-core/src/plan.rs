use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanAction {
    Create { relative_path: PathBuf },
    Update { relative_path: PathBuf },
    Archive { relative_path: PathBuf },
    Skip { relative_path: PathBuf },
    Conflict { relative_path: PathBuf, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub profile_id: String,
    pub source_vault: PathBuf,
    pub destination_vault: PathBuf,
    pub actions: Vec<PlanAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunReport {
    pub profile_id: String,
    pub created_count: usize,
    pub updated_count: usize,
    pub archived_count: usize,
    pub skipped_count: usize,
    pub conflict_count: usize,
}
