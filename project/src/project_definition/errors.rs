use std::io;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum FailedToOpenProjectError {
    #[error("Failed to open project file for loading: {error}")]
    FailedToOpenFile { error: String },
    #[error("Failed to read project file: {error}")]
    FailedToReadFile { error: String },
    #[error("Failed to parse project file: {error}")]
    FailedToParseFile { error: String },
}

impl From<io::Error> for FailedToOpenProjectError {
    fn from(error: io::Error) -> Self {
        FailedToOpenProjectError::FailedToOpenFile {
            error: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for FailedToOpenProjectError {
    fn from(error: serde_json::Error) -> Self {
        FailedToOpenProjectError::FailedToParseFile {
            error: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum FailedToSaveProjectError {
    #[error("Failed to open project file for saving: {error}")]
    FailedToOpenFile { error: String },
    #[error("Failed to write to project file: {error}")]
    FailedToParseProjectObject { error: String },
}

impl From<io::Error> for FailedToSaveProjectError {
    fn from(error: io::Error) -> Self {
        FailedToSaveProjectError::FailedToOpenFile {
            error: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for FailedToSaveProjectError {
    fn from(error: serde_json::Error) -> Self {
        FailedToSaveProjectError::FailedToParseProjectObject {
            error: error.to_string(),
        }
    }
}
