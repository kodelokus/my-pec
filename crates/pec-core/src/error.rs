use serde::Serialize;
use ts_rs::TS;

/// Unexpected failures returned by commands. Expected outcomes are values in each command's result type.
#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum AppError {
    #[error("cannot write {path}")]
    WorkspaceNotWritable { path: String },
    #[error("invalid {field}")]
    InvalidInput { field: String },
    #[error("{message}")]
    Internal { message: String },
}

impl AppError {
    pub fn invalid(field: &str) -> Self {
        Self::InvalidInput {
            field: field.into(),
        }
    }

    pub fn not_writable(path: &std::path::Path) -> Self {
        Self::WorkspaceNotWritable {
            path: path.display().to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
