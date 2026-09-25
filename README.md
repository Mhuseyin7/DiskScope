# DiskScope

> Local-first disk usage analyzer, storage explorer and safe cleanup assistant.

**DiskScope açık kaynaklı bir projedir ve [muhammedkoca.com.tr](https://muhammedkoca.com.tr) tarafından geliştirilmiştir.**

DiskScope bir “one-click cleaner” değildir. Temel prensibi: **Önce inspect et. Sonra açıkla. Sadece kullanıcı açıkça onayladıktan sonra temizle.**

Windows, macOS ve Linux kullanıcılarının disk space kullanımını anlamasına; büyük files, developer cache'leri, duplicate content ve güvenli cleanup fırsatlarını incelemesine yardımcı olur.

## Neden DiskScope?

Docker image'ları, `node_modules`, package cache'leri, eski download'lar, logs ve duplicate media zamanla görünmez biçimde disk space tüketir. DiskScope sadece boyut göstermez; her classification veya cleanup recommendation için **nedenini**, **risk level'ını** ve mümkünse **undo durumunu** görünür kılmayı hedefler.

## Mevcut özellikler

İlk release foundation şu özellikleri içerir:

- Local folder seçimi ile disk scan başlatma
- Streaming progress, responsive UI ve scan cancellation
- Bounded-memory filesystem traversal
- Symlink'leri takip etmeme; permission error'ları gizlemek yerine raporlama
- Explainable category classification ve category bazlı usage summary
- Exact duplicate detection pipeline: file size → partial BLAKE3 → full SHA-256
- Güvenli cleanup primitives: selected-root boundary, symlink ve high-risk validation
- Uygun hedeflerde system Trash / Recycle Bin abstraction

> Docker inspection, snapshot comparison, duplicate browser UI, platform-specific cache discovery ve installer release pipeline roadmap içindedir. Uygulama, tamamlanmamış özellikleri tamamlanmış gibi göstermez.

## Safety model

- Default olarak tamamen local-first çalışır; account, cloud dependency veya filename telemetry yoktur.
- Scan sırasında symlink'ler normal folder gibi traverse edilmez.
- Inaccessible path'ler silent olarak ignore edilmez; kullanıcıya raporlanır.
- Cleanup öncesinde target yeniden validate edilir.
- Selected root dışındaki hedefler, symlink'ler ve `HIGH RISK` generic cleanup flow tarafından reddedilir.
- Mümkün olduğunda permanent deletion yerine system Trash / Recycle Bin kullanılır.

## Technology stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri 2 |
| Frontend | React 19 + TypeScript strict mode |
| Build tool | Vite |
| Core engine | Rust |
| Filesystem traversal | `walkdir` |
| Duplicate sampling | BLAKE3 |
| Exact confirmation | SHA-256 |
| Quality checks | ESLint, TypeScript, Clippy |

## Project structure

```text
DiskScope/
├── apps/desktop/             # Tauri + React desktop application
├── crates/
│   ├── common/               # Shared serializable domain types
│   ├── scanner/              # Streaming filesystem scanner
│   ├── analyzer/             # Explainable category rules
│   ├── duplicates/           # Staged exact duplicate detection
│   └── cleanup/              # Guarded cleanup and history primitives
├── docs/                     # Architecture and roadmap
├── fixtures/                 # Safe test fixture guidance
└── .github/workflows/        # CI checks
```

## Quick start

### Requirements

- [Rust stable](https://www.rust-lang.org/tools/install)
- Node.js 24 veya daha yeni bir sürüm
- Windows için Tauri C++ Build Tools ve WebView2 Runtime

### Development

```bash
git clone https://github.com/Mhuseyin7/DiskScope.git
cd DiskScope/apps/desktop
npm install
npm run dev:desktop
```

Sadece frontend preview için `npm run dev` çalıştırabilirsiniz.

### Quality checks

```bash
# Repository root
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# Frontend
cd apps/desktop
npm run lint
npm run build
```

## Roadmap

- Large file ve directory explorer
- Intelligent treemap visualization
- Developer cache discovery (`npm`, `pnpm`, Cargo, Python, Gradle ve daha fazlası)
- Docker image, container, build cache ve volume analysis
- Reviewed cleanup recommendations with transparent risk levels
- Exact duplicate browser ve safe user selection
- Metadata snapshot comparison ve opt-in watch mode
- Signed installers, checksums ve release automation

Detaylı plan: [docs/ROADMAP.md](docs/ROADMAP.md)

## Contributing

Katkılar memnuniyetle karşılanır. Pull request öncesinde format, lint, test ve build süreçlerini çalıştırın. Cleanup ile ilgili her değişiklik risk level, reversibility ve path validation davranışını açıkça açıklamalıdır.

Detaylar: [CONTRIBUTING.md](CONTRIBUTING.md)

## Security & privacy

- [Security policy](SECURITY.md)
- [Privacy approach](PRIVACY.md)
- [Technical architecture](ARCHITECTURE.md)
- [Changelog](CHANGELOG.md)

## License

Bu proje açık kaynaklıdır ve [MIT License](LICENSE) ile lisanslanmıştır.

---

Built with care by [muhammedkoca.com.tr](https://muhammedkoca.com.tr) · Local-first storage intelligence for people who want to understand before they delete.
