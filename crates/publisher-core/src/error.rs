use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Unsafe path: {0}")]
    UnsafePath(String),
}
