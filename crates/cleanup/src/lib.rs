//! Cleanup protection layer. Callers must explicitly confirm each generated plan.
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use diskscope_common::{CleanupCandidate, RiskLevel};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRecord {
    pub timestamp: DateTime<Utc>,
    pub paths: Vec<PathBuf>,
    pub bytes: u64,
    pub method: String,
    pub undo_available: bool,
}
pub fn validate(candidate: &CleanupCandidate, allowed_roots: &[PathBuf]) -> Result<()> {
    if candidate.risk == RiskLevel::HighRisk {
        bail!("high-risk targets require a dedicated review flow")
    }
    let metadata = fs::symlink_metadata(&candidate.path).context("target no longer exists")?;
    if metadata.file_type().is_symlink() {
        bail!("refusing to clean a symbolic link")
    }
    let path = candidate.path.canonicalize()?;
    if !allowed_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| path.starts_with(root))
    {
        bail!("target is outside the selected scan root")
    }
    Ok(())
}
pub fn move_to_trash(
    candidate: &CleanupCandidate,
    allowed_roots: &[PathBuf],
) -> Result<CleanupRecord> {
    validate(candidate, allowed_roots)?;
    trash::delete(&candidate.path).context("could not move target to system trash")?;
    Ok(CleanupRecord {
        timestamp: Utc::now(),
        paths: vec![candidate.path.clone()],
        bytes: candidate.estimated_bytes,
        method: "system-trash".into(),
        undo_available: true,
    })
}
pub fn persist_history(history_file: &Path, record: &CleanupRecord) -> Result<()> {
    let mut history: Vec<CleanupRecord> = fs::read(history_file)
        .ok()
        .and_then(|x| serde_json::from_slice(&x).ok())
        .unwrap_or_default();
    history.push(record.clone());
    if let Some(parent) = history_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(history_file, serde_json::to_vec_pretty(&history)?)?;
    Ok(())
}
