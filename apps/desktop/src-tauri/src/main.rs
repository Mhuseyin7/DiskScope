#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use diskscope_analyzer::classify;
use diskscope_common::{FileEntry, ScanProgress};
use diskscope_scanner::{CancelToken, ScanOptions, scan};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};

struct ScanState(Arc<Mutex<Option<CancelToken>>>);
#[derive(Serialize)]
struct ScanSummary {
    entries: u64,
    bytes: u64,
    errors: usize,
    cancelled: bool,
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
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let mut entries = 0;
        let mut categories = BTreeMap::new();
        let outcome = scan(
            &root,
            &options,
            &token,
            |mut entry: FileEntry| {
                let rule = classify(&entry.path);
                entry.category = rule.category;
                *categories.entry(entry.category).or_insert(0) += entry.size;
                entries += 1;
            },
            |progress| emit_progress(&app, progress),
        );
        (outcome, entries, categories)
    })
    .await
    .map_err(|_| "scan worker stopped unexpectedly")?;
    *state.0.lock().map_err(|_| "scan state unavailable")? = None;
    Ok(ScanSummary {
        entries: outcome.1,
        bytes: outcome.0.progress.bytes_seen,
        errors: outcome.0.errors.len(),
        cancelled: outcome.0.cancelled,
        categories: outcome.2,
    })
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ScanState(Arc::new(Mutex::new(None))))
        .invoke_handler(tauri::generate_handler![scan_folder, cancel_scan])
        .run(tauri::generate_context!())
        .expect("error while running DiskScope");
}
