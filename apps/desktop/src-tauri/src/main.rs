#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use diskscope_analyzer::classify;
use diskscope_cleanup::{CleanupRecord, move_to_trash, persist_history};
use diskscope_common::{CleanupCandidate, FileEntry, RiskLevel, ScanProgress};
use diskscope_duplicates::find_exact;
use diskscope_scanner::{CancelToken, ScanOptions, scan};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};

struct ScanState(Arc<Mutex<Option<CancelToken>>>);
struct ScanCache(Mutex<Option<ScanCacheValue>>);
#[derive(Clone)]
struct ScanCacheValue {
    root: PathBuf,
    files: Vec<FileEntry>,
    candidates: Vec<CleanupView>,
}
#[derive(Clone, Serialize, Deserialize)]
struct ScanSummary {
    entries: u64,
    bytes: u64,
    errors: usize,
    cancelled: bool,
    categories: BTreeMap<String, u64>,
    large_files: Vec<FileView>,
    development: Vec<CleanupView>,
}
#[derive(Clone, Serialize, Deserialize)]
struct FileView {
    path: String,
    size: u64,
    modified: Option<String>,
    category: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct CleanupView {
    id: String,
    path: String,
    estimated_bytes: u64,
    title: String,
    reason: String,
    risk: String,
    reversible: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct DuplicateView {
    bytes_per_file: u64,
    files: Vec<String>,
    hash: String,
}
#[derive(Serialize)]
struct DuplicateResult {
    groups: Vec<DuplicateView>,
    limited: bool,
}
#[derive(Serialize)]
struct DockerResult {
    available: bool,
    detail: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Snapshot {
    timestamp: u64,
    root: String,
    bytes: u64,
    entries: u64,
    categories: BTreeMap<String, u64>,
}
fn emit_progress(app: &AppHandle, progress: &ScanProgress) {
    let _ = app.emit("scan-progress", progress);
}
#[tauri::command]
fn cancel_scan(state: State<'_, ScanState>) -> Result<(), String> {
    if let Some(token) = state
        .0
        .lock()
        .map_err(|_| "scan state unavailable")?
        .as_ref()
    {
        token.cancel();
    }
    Ok(())
}
#[tauri::command]
async fn scan_folder(
    app: AppHandle,
    state: State<'_, ScanState>,
    cache: State<'_, ScanCache>,
    root: String,
    mode: String,
) -> Result<ScanSummary, String> {
    let root = PathBuf::from(root);
    if !root.is_dir() {
        return Err("The chosen path is not a directory".into());
    }
    let token = CancelToken::default();
    {
        let mut active = state.0.lock().map_err(|_| "scan state unavailable")?;
        if active.is_some() {
            return Err("A scan is already running".into());
        }
        *active = Some(token.clone());
    }
    let options = ScanOptions {
        include_hidden: true,
        max_depth: if mode == "quick" { Some(4) } else { None },
    };
    let scan_root = root.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut entries = 0;
        let mut categories = BTreeMap::new();
        let mut files = Vec::new();
        let mut large_files = Vec::new();
        let mut development: HashMap<PathBuf, (u64, String, RiskLevel, String)> = HashMap::new();
        let outcome = scan(
            &scan_root,
            &options,
            &token,
            |mut entry: FileEntry| {
                let rule = classify(&entry.path);
                entry.category = rule.category;
                *categories.entry(entry.category.clone()).or_insert(0) += entry.size;
                entries += 1;
                if !entry.is_dir && entry.size > 0 {
                    if large_files.len() < 200
                        || entry.size
                            > large_files
                                .last()
                                .map(|item: &FileView| item.size)
                                .unwrap_or(0)
                    {
                        large_files.push(file_view(&entry));
                        large_files.sort_by_key(|item| std::cmp::Reverse(item.size));
                        large_files.truncate(200);
                    }
                    if files.len() < 100_000 {
                        files.push(entry.clone());
                    }
                    if let Some((path, title, risk, reason)) = cache_candidate(&entry.path) {
                        let item = development.entry(path).or_insert((0, title, risk, reason));
                        item.0 += entry.size;
                    }
                }
            },
            |progress| emit_progress(&app, progress),
        );
        let mut candidates: Vec<CleanupView> = development
            .into_iter()
            .map(
                |(path, (estimated_bytes, title, risk, reason))| CleanupView {
                    id: path.to_string_lossy().to_string(),
                    path: path.to_string_lossy().to_string(),
                    estimated_bytes,
                    title,
                    reason,
                    risk: risk_label(risk).into(),
                    reversible: true,
                },
            )
            .collect();
        candidates.sort_by_key(|item| std::cmp::Reverse(item.estimated_bytes));
        (outcome, entries, categories, files, large_files, candidates)
    })
    .await
    .map_err(|_| "scan worker stopped unexpectedly")?;
    *state.0.lock().map_err(|_| "scan state unavailable")? = None;
    let summary = ScanSummary {
        entries: result.1,
        bytes: result.0.progress.bytes_seen,
        errors: result.0.errors.len(),
        cancelled: result.0.cancelled,
        categories: result.2,
        large_files: result.4.clone(),
        development: result.5.clone(),
    };
    *cache.0.lock().map_err(|_| "scan cache unavailable")? = Some(ScanCacheValue {
        root,
        files: result.3,
        candidates: result.5,
    });
    Ok(summary)
}
fn file_view(entry: &FileEntry) -> FileView {
    FileView {
        path: entry.path.to_string_lossy().to_string(),
        size: entry.size,
        modified: entry.modified.map(|x| x.to_rfc3339()),
        category: entry.category.clone(),
    }
}
fn risk_label(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "SAFE",
        RiskLevel::LowRisk => "LOW RISK",
        RiskLevel::Review => "REVIEW",
        RiskLevel::HighRisk => "HIGH RISK",
    }
}
fn cache_candidate(path: &Path) -> Option<(PathBuf, String, RiskLevel, String)> {
    let components: Vec<_> = path.components().collect();
    for (index, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy().to_lowercase();
        let (title, risk, reason) = match name.as_str() {
            "node_modules" => (
                "node_modules",
                RiskLevel::Review,
                "Project dependencies can be restored, but the next install/build may take time.",
            ),
            ".next" | ".vite" | "__pycache__" => (
                "Build cache",
                RiskLevel::Safe,
                "Regenerable build or interpreter cache.",
            ),
            "target" => (
                "Rust target directory",
                RiskLevel::LowRisk,
                "Regenerable Rust build output.",
            ),
            ".cache" => (
                "Application cache",
                RiskLevel::LowRisk,
                "Application cache may be rebuilt after cleanup.",
            ),
            _ => continue,
        };
        let root = components[..=index].iter().collect::<PathBuf>();
        return Some((root, title.into(), risk, reason.into()));
    }
    None
}
#[tauri::command]
async fn find_duplicates(cache: State<'_, ScanCache>) -> Result<DuplicateResult, String> {
    let value = cache
        .0
        .lock()
        .map_err(|_| "scan cache unavailable")?
        .clone()
        .ok_or("Run a scan before looking for duplicates")?;
    let limited = value.files.len() == 100_000;
    let groups = tauri::async_runtime::spawn_blocking(move || {
        find_exact(value.files).map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| "duplicate worker stopped unexpectedly")??
    .into_iter()
    .map(|group| DuplicateView {
        bytes_per_file: group.bytes_per_file,
        files: group
            .files
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        hash: group.hash,
    })
    .collect();
    Ok(DuplicateResult { groups, limited })
}
#[tauri::command]
fn docker_status() -> DockerResult {
    match Command::new("docker").args(["system", "df", "-v"]).output() {
        Ok(output) if output.status.success() => DockerResult {
            available: true,
            detail: String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(12_000)
                .collect(),
        },
        Ok(output) => DockerResult {
            available: false,
            detail: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(_) => DockerResult {
            available: false,
            detail: "Docker CLI was not found or is not available to DiskScope.".into(),
        },
    }
}
#[tauri::command]
fn cleanup_candidate(
    app: AppHandle,
    cache: State<'_, ScanCache>,
    id: String,
) -> Result<CleanupRecord, String> {
    let value = cache
        .0
        .lock()
        .map_err(|_| "scan cache unavailable")?
        .clone()
        .ok_or("Run a scan before cleanup")?;
    let item = value
        .candidates
        .into_iter()
        .find(|candidate| candidate.id == id)
        .ok_or("Cleanup target is no longer available")?;
    let risk = match item.risk.as_str() {
        "SAFE" => RiskLevel::Safe,
        "LOW RISK" => RiskLevel::LowRisk,
        _ => RiskLevel::Review,
    };
    let candidate = CleanupCandidate {
        path: PathBuf::from(item.path),
        estimated_bytes: item.estimated_bytes,
        title: item.title,
        reason: item.reason,
        risk,
        reversible: item.reversible,
    };
    let record = move_to_trash(&candidate, &[value.root]).map_err(|error| error.to_string())?;
    let history = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("cleanup-history.json");
    persist_history(&history, &record).map_err(|error| error.to_string())?;
    Ok(record)
}
#[tauri::command]
fn save_snapshot(app: AppHandle, summary: ScanSummary, root: String) -> Result<Snapshot, String> {
    let snapshot = Snapshot {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        root,
        bytes: summary.bytes,
        entries: summary.entries,
        categories: summary.categories,
    };
    let file = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("snapshots.json");
    let mut snapshots: Vec<Snapshot> = std::fs::read(&file)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default();
    snapshots.push(snapshot.clone());
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(
        file,
        serde_json::to_vec_pretty(&snapshots).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(snapshot)
}
#[tauri::command]
fn cleanup_history(app: AppHandle) -> Result<Vec<CleanupRecord>, String> {
    let file = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("cleanup-history.json");
    Ok(std::fs::read(file)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ScanState(Arc::new(Mutex::new(None))))
        .manage(ScanCache(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            cancel_scan,
            find_duplicates,
            docker_status,
            cleanup_candidate,
            save_snapshot,
            cleanup_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running DiskScope");
}
