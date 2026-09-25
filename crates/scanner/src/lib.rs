//! Bounded-memory filesystem traversal. Symlinks are reported but never traversed.
use chrono::{DateTime, Utc};
use diskscope_common::{FileEntry, ScanError, ScanProgress};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use walkdir::WalkDir;

#[derive(Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);
impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release)
    }
    pub fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
}
impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_hidden: true,
            max_depth: None,
        }
    }
}

pub struct ScanOutcome {
    pub progress: ScanProgress,
    pub errors: Vec<ScanError>,
    pub cancelled: bool,
}

pub fn scan<F, P>(
    root: &Path,
    options: &ScanOptions,
    cancelled: &CancelToken,
    mut on_entry: F,
    mut on_progress: P,
) -> ScanOutcome
where
    F: FnMut(FileEntry),
    P: FnMut(&ScanProgress),
{
    let mut progress = ScanProgress::default();
    let mut errors = Vec::new();
    let mut walker = WalkDir::new(root)
        .follow_links(false)
        .same_file_system(false)
        .into_iter();
    while let Some(item) = walker.next() {
        if cancelled.cancelled() {
            break;
        }
        let entry = match item {
            Ok(value) => value,
            Err(error) => {
                let path = error.path().unwrap_or(root).to_path_buf();
                progress.inaccessible_paths += 1;
                errors.push(ScanError {
                    path,
                    message: error.to_string(),
                });
                continue;
            }
        };
        if entry.depth() > 0 && !options.include_hidden && is_hidden(entry.path()) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }
        if options.max_depth.is_some_and(|limit| entry.depth() > limit) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(value) => value,
            Err(error) => {
                progress.inaccessible_paths += 1;
                errors.push(ScanError {
                    path: entry.path().to_path_buf(),
                    message: error.to_string(),
                });
                continue;
            }
        };
        let size = if metadata.is_file() {
            metadata.len()
        } else {
            0
        };
        progress.entries_seen += 1;
        progress.bytes_seen += size;
        progress.current_path = Some(entry.path().to_path_buf());
        on_entry(FileEntry {
            path: entry.path().to_path_buf(),
            size,
            modified: metadata.modified().ok().map(DateTime::<Utc>::from),
            is_dir: metadata.is_dir(),
            is_symlink: entry.file_type().is_symlink(),
            category: String::new(),
        });
        if progress.entries_seen % 256 == 0 {
            on_progress(&progress);
        }
    }
    on_progress(&progress);
    ScanOutcome {
        progress,
        errors,
        cancelled: cancelled.cancelled(),
    }
}
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_starts_active() {
        assert!(!CancelToken::default().cancelled());
    }
}
