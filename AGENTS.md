## Current State (Jun 1, 2026)

### Slint Migration - Complete
- Replaced Tauri+Nuxt frontend with Slint native GUI
- `crates/kcraft-gui/` now contains Slint-based app (no npm/Node.js deps)
- Old `frontend/` and `src-tauri/` directories removed
- CI/release workflows updated to remove npm/Tauri steps

### Key design decisions
- `Weak<AppWindow>` is `Send` — used in spawned threads with `slint::invoke_from_event_loop`
- MSA flow runs in a separate thread to avoid blocking UI
- Net events subscribed in a dedicated thread, updates UI via `invoke_from_event_loop`
- `rfd` crate for native file dialogs instead of Tauri's dialog API

### Files changed
- `crates/kcraft-gui/Cargo.toml` — slint, rfd, tokio deps
- `crates/kcraft-gui/build.rs` — `slint_build::compile("ui/app.slint")`
- `crates/kcraft-gui/ui/app.slint` — full UI layout (502 lines)
- `crates/kcraft-gui/src/main.rs` — entry point (~30 lines)
- `crates/kcraft-gui/src/data_root.rs` — path utilities
- `crates/kcraft-gui/src/accounts.rs` — account model, add offline/elyby
- `crates/kcraft-gui/src/msa.rs` — MSA auth (threaded, catch_unwind)
- `crates/kcraft-gui/src/instances.rs` — instance model, refresh, launch (threaded)
- `crates/kcraft-gui/src/modpack.rs` — modpack builder (honest messaging)
- `crates/kcraft-gui/src/net_events.rs` — net event subscription thread
- `.github/workflows/ci.yml` — removed Node/Nuxt steps, simplified deps
- `.github/workflows/release.yml` — removed Tauri action, added cargo build + upload
- Removed: `crates/kcraft-gui/frontend/`, `crates/kcraft-gui/src-tauri/`

### CI checks
- `cargo fmt --all -- --check` — passes
- `cargo clippy --workspace --all-targets -- -D warnings` — passes (0 warnings)
- `cargo test --workspace` — passes
- `cargo build --release -p kcraft-gui` — builds (35 MB binary)

### Issues fixed during audit
- Removed unused `ProgressBanner` component (dead code)
- Fixed progress bar width formula (`completed * 100% / total * 1px` restored — Slint needs `1px` for float→length conversion)
- Fixed progress text (`completed / 1000` → `completed` — was dividing action counts)
- Fixed `data_root()` using `kcrack` instead of `kcraft` (directory name typo)
- Moved instance launch to spawned thread (was blocking UI)
- Added `catch_unwind` to MSA flow (prevents overlay being stuck)
- Fixed modpack builder message: "Installed X packages" → "Added X file(s)"
- Fixed `padding-top` on non-layout element
- Split monolithic main.rs into modular structure (6 modules)
- Removed unused deps: `serde_json`, `kcraft-config`

## Roadmap

### Sprint 1 — Foundation (COMPLETE)
- [x] Replace Tauri+Nuxt with Slint native GUI
- [x] Modular project structure (accounts, msa, instances, modpack, net_events)
- [x] Wire all callbacks to Slint UI
- [x] CI/CD updated (no npm/Tauri deps)
- [x] All checks pass (fmt, clippy -D warnings, test, release build)

### Sprint 2 — Stability & UX
- [ ] Graceful shutdown: stop net thread on app exit (signal via `Drop` or channel)
- [ ] Loading states: show progress/spinner during instance list load
- [ ] Error recovery: retry button when instance list fails
- [ ] Confirm dialogs before destructive actions (delete account/instance)
- [ ] Display account UUID and token status in UI
- [ ] Instance details view (java path, RAM settings, mod list)

### Sprint 3 — Instance Management
- [ ] Create new instance dialog (name, MC version, modloader)
- [ ] Delete/duplicate/rename instances
- [ ] Edit instance settings (java args, RAM, resolution)
- [ ] Instance status indicator (not launched, running, crashed)
- [ ] Instance launch progress bar (show launch step names)

### Sprint 4 — Modpack Builder
- [ ] File dependency parsing (read `fabric.mod.json`, `META-INF/MANIFEST.MF`)
- [ ] Actual JAR extraction and analysis
- [ ] Conflict resolution display (show conflicting mods)
- [ ] Modpack export to CurseForge / mrpack format
- [ ] Mod file management (enable/disable mods)

### Sprint 5 — Account Management
- [ ] Token refresh (silent background refresh for MSA)
- [ ] Delete accounts from UI
- [ ] Set default account
- [ ] Display token expiry/validity
- [ ] Multiple profiles per MSA account
- [ ] Head (skin) display for accounts

### Sprint 6 — Polish & Distribution
- [ ] Dark/light theme toggle
- [ ] App icon and desktop file
- [ ] Flatpak/AppImage packaging
- [ ] Auto-updater integration
- [ ] Crash reporter with panic hook → log file
- [ ] Translation/i18n support
