//! Shared, serialisable domain types. Paths are deliberately never converted into shell commands.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Safe,
    LowRisk,
    Review,
    HighRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanProgress {
    pub entries_seen: u64,
    pub bytes_seen: u64,
    pub current_path: Option<PathBuf>,
    pub inaccessible_paths: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub path: PathBuf,
    pub estimated_bytes: u64,
    pub title: String,
    pub reason: String,
    pub risk: RiskLevel,
    pub reversible: bool,
}
