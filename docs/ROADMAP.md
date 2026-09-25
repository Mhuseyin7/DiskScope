# Delivery roadmap

## Implemented foundation

- Tauri desktop shell with accessible navigation and theme-aware system-utility UI.
- Folder selection, streamed scan progress, cancellation, category aggregation, and reported permission failures.
- Bounded-memory traversal that does not follow symlinks.
- Explainable initial category rules.
- Exact-duplicate pipeline (size, sample BLAKE3, full SHA-256).
- Guarded cleanup primitives that reject symlinks, enforce selected-root containment, reject high-risk generic cleanup, and use system Trash/Recycle Bin.

## Next release milestones

1. Persist streamed entries to a local SQLite snapshot store; add treemap, folder, large-file, search, and snapshot comparison views.
2. Add fixture-driven scanner and cleanup integration tests, including inaccessible paths, deleted-during-scan files, and Windows reparse points.
3. Expand platform adapters for development caches, Docker's supported APIs, Recycle Bin, and mount points.
4. Add reviewed cleanup recommendations, explicit confirmation UI, durable cleanup history, and per-platform revalidation hardening.
5. Add package targets, signed release process, checksums, and Tauri-compatible end-to-end tests.

No milestone should promote a cleanup target to SAFE without a documented rule, a recovery path where supported, and fixture coverage.
