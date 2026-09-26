export type RiskLevel = "SAFE" | "LOW RISK" | "REVIEW" | "HIGH RISK";
export interface ScanProgress { entries_seen: number; bytes_seen: number; current_path?: string; inaccessible_paths: number }
export interface FileView { path: string; size: number; modified?: string; category: string }
export interface CleanupCandidate { id: string; path: string; estimated_bytes: number; title: string; reason: string; risk: RiskLevel; reversible: boolean }
export interface ScanSummary { entries: number; bytes: number; errors: number; cancelled: boolean; categories: Record<string, number>; large_files: FileView[]; development: CleanupCandidate[] }
export interface DuplicateGroup { bytes_per_file: number; hash: string; files: string[] }
export interface DuplicateResult { groups: DuplicateGroup[]; limited: boolean }
export interface DockerResult { available: boolean; detail: string }
export interface CleanupRecord { timestamp: string; paths: string[]; bytes: number; method: string; undo_available: boolean }
