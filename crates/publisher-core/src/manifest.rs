use crate::error::PublisherError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const MANIFEST_DIR: &str = ".vault-publisher";
pub const MANIFEST_FILENAME: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: u32,
    pub profile_id: String,
    pub last_run: String,
    pub files: HashMap<String, FileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub relative_path: String,
    pub source_hash: String,
    pub destination_hash: String,
    pub last_published: String,
}

impl Manifest {
    pub fn new(profile_id: &str) -> Self {
        Self {
            version: 1,
            profile_id: profile_id.to_string(),
            last_run: String::new(),
            files: HashMap::new(),
        }
    }
}

pub fn compute_sha256(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub fn compute_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    Ok(compute_sha256(&bytes))
}

pub fn load_manifest(dest_vault: &Path) -> Result<Manifest, PublisherError> {
    let manifest_path = dest_vault.join(MANIFEST_DIR).join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Err(PublisherError::InvalidPath(
            "Manifest file does not exist".to_string(),
        ));
    }
    let data = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&data)?;
    Ok(manifest)
}

pub fn save_manifest(dest_vault: &Path, manifest: &Manifest) -> Result<(), PublisherError> {
    let dir = dest_vault.join(MANIFEST_DIR);
    fs::create_dir_all(&dir)?;

    let manifest_path = dir.join(MANIFEST_FILENAME);
    let tmp_path = dir.join(format!("{}.tmp", MANIFEST_FILENAME));

    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, &manifest_path)?;

    Ok(())
}
