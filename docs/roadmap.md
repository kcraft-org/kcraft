# kcraft Roadmap

This file is a historical record. Completed items are marked with `[done]` and preserved.

## v1.0.0 (2026-06-01)

[done] Replace Tauri+Nuxt with Slint native GUI. [done] Modular project structure (accounts, msa, instances, modpack, net_events). [done] Wire all callbacks to Slint UI. [done] CI/CD updated (no npm/Tauri deps). [done] All checks pass (fmt, clippy -D warnings, test, release build). [done] Graceful shutdown: stop net thread on app exit. [done] Confirm dialogs before destructive actions. [done] Account UUID and token validity display. [done] Instance details view. [done] Create/delete/rename/duplicate instances. [done] Edit instance settings (java, RAM, resolution, JVM args). [done] Instance launch progress tracking. [done] Modpack builder with JAR parsing, conflict detection, file management. [done] MSA token refresh. [done] Delete account, set default account. [done] Theme toggle (dark/light). [done] Crash reporter with log file. [done] Flatpak packaging support. [done] App icon and desktop file. [done] DAG-based dependency resolver.

## Future

Planned future work includes instance status indicator (not launched, running, crashed), modpack export to CurseForge / mrpack format, multiple profiles per MSA account, skin display for accounts, auto-updater integration, translation/i18n support, and ARM Linux support.
