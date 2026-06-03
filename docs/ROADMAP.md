# kcraft Roadmap

This file is a historical record. Completed items are marked with `[done]` and preserved.

## v1.0.0 (2026-06-01)

[done] Replace Tauri+Nuxt with Slint native GUI. [done] Modular project structure (accounts, msa, instances, modpack, net_events). [done] Wire all callbacks to Slint UI. [done] CI/CD updated (no npm/Tauri deps). [done] All checks pass (fmt, clippy -D warnings, test, release build). [done] Graceful shutdown: stop net thread on app exit. [done] Confirm dialogs before destructive actions. [done] Account UUID and token validity display. [done] Instance details view. [done] Create/delete/rename/duplicate instances. [done] Edit instance settings (java, RAM, resolution, JVM args). [done] Instance launch progress tracking. [done] Modpack builder with JAR parsing, conflict detection, file management. [done] MSA token refresh. [done] Delete account, set default account. [done] Theme toggle (dark/light). [done] Crash reporter with log file. [done] Flatpak packaging support. [done] App icon and desktop file. [done] DAG-based dependency resolver.

## Sprint: Feature Parity & Quality (v1.1.0)

After auditing the launcher landscape and identifying gaps, we target the following priorities:

### P0 — Ship-blocking
- [ ] Auto Java management: per-instance Java auto-detection and auto-download
- [ ] Mod auto-update from CurseForge and Modrinth
- [ ] Instance cloning (copy full instance with all config)
- [ ] Crash reporter with structured log capture and stack trace
- [ ] Built-in self-updater (binary replacement on launch)

### P1 — Competitive parity
- [ ] World management panel with restore from backup
- [ ] Resource pack browser and one-click install
- [ ] Shader pack installation from supported platforms
- [ ] Per-instance JVM args UI (memory, GC, custom flags)
- [ ] Portable mode (all data relative to binary path)
- [ ] Custom theme support via Slint theming API

### P2 — Advantage features
- [ ] Pre-launch and post-launch shell script hooks per instance
- [ ] Quilt mod loader support
- [ ] FTB and Technic modpack import
- [ ] Server pack export from client instance
- [ ] Cloud sync / instance sharing via URL

### P3 — Aspirational
- [ ] PvP overlay (FPS, CPS, coordinates, keystrokes)
- [ ] Controller support via SDL2 binding
- [ ] Mobile build target (Android/iOS via Slint)
- [ ] Plugin/extension API for community modules
- [ ] Instance marketplace / modpack browser inside launcher

## Future

Planned future work includes instance status indicator (not launched, running, crashed), modpack export to CurseForge / mrpack format, multiple profiles per MSA account, skin display for accounts, auto-updater integration, translation/i18n support, and ARM Linux support.
