export type RiskLevel = "SAFE" | "LOW_RISK" | "REVIEW" | "HIGH_RISK";
export interface ScanProgress { entries_seen: number; bytes_seen: number; current_path?: string; inaccessible_paths: number }
export interface ScanSummary { entries: number; bytes: number; errors: number; cancelled: boolean; categories: Record<string, number> }
export interface CleanupCandidate { path: string; estimated_bytes: number; title: string; reason: string; risk: RiskLevel; reversible: boolean }
