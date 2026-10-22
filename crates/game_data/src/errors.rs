use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("{kind} '{id}' not found")]
    NotFound { kind: &'static str, id: String },

    #[error("could not read or write {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid JSON in {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("validation failed: {0}")]
    Invalid(String),
}
