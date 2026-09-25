//! Explainable, path-based classification. Rules return their rationale to the UI.
use diskscope_common::FileEntry;
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Serialize)]
pub struct Classification {
    pub category: String,
    pub reason: String,
}
pub fn classify(path: &Path) -> Classification {
    let lower = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let rules = [
        ("Docker", "docker", "Path is inside a Docker data directory"),
        ("Development", "node_modules", "Node.js dependencies"),
        ("Development", ".pnpm-store", "pnpm package store"),
        ("Development", ".cargo/registry", "Cargo package cache"),
        ("Development", "target", "Rust build output"),
        ("Caches", ".cache", "Application cache directory"),
        ("Temporary files", "/temp/", "Temporary-file directory"),
        ("Logs", "/logs/", "Log directory"),
        ("Downloads", "/downloads/", "Downloads directory"),
    ];
    for (category, marker, reason) in rules {
        if lower.contains(marker) {
            return Classification {
                category: category.into(),
                reason: reason.into(),
            };
        }
    }
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_lowercase();
    let category = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "mp4" | "mov" | "mkv" => "Media",
        "zip" | "7z" | "rar" | "tar" | "gz" => "Archives",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" => "Documents",
        _ => "Unknown",
    };
    Classification {
        category: category.into(),
        reason: "No specific rule matched; classified from extension when possible".into(),
    }
}
pub fn breakdown(entries: impl IntoIterator<Item = FileEntry>) -> BTreeMap<String, u64> {
    let mut result = BTreeMap::new();
    for entry in entries {
        *result.entry(entry.category).or_default() += entry.size;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_node_modules() {
        assert_eq!(
            classify(Path::new("x/node_modules/a")).category,
            "Development"
        );
    }
}
