# Architecture

`apps/desktop` is a Tauri 2 desktop client. React owns presentation and calls narrow Rust commands; it does not obtain broad filesystem access.

Rust crates are deliberately separated:

- `scanner`: streaming traversal, cancellation, progress, error collection.
- `analyzer`: explainable rule-based categories and aggregations.
- `duplicates`: size → sample BLAKE3 → full SHA-256 exact matching.
- `cleanup`: revalidation, root-boundary checks, Trash/Recycle Bin abstraction, history persistence.
- `common`: serialisable contracts shared across layers.

The current scanner uses bounded working memory for traversal; consumers may stream results to aggregates or persistent snapshots instead of retaining an entire tree. Symlinks are observed but never followed. Platform-specific discovery and a least-privilege helper belong in separate platform adapters before privileged features are introduced.
