# Changelog

## v1.0.0 (2026-06-01)

Initial release of KCraft - a next-generation Minecraft launcher built with Rust and Tauri.

### Features

- Minecraft instance management with full launch pipeline
- Microsoft MSA authentication (device code flow)
- Offline (cracked) account support
- Ely.by / Authlib-Injector authentication
- Mod platform integration: CurseForge, Modrinth, FTB, ATLauncher, Technic, Packwiz
- DAG-based dependency resolver for modpack building
- Async download engine with resume, validation, and progress reporting
- Java runtime auto-detection (Linux, macOS, Windows)
- Nuxt/Vue 3 glassmorphism UI with Tauri 2 desktop shell
- Self-updating via Tauri updater plugin
- Cross-platform: Linux, Windows, macOS

### Bug Fixes

- Fixed volatile `/tmp/kcrack` data fallback to use stable home directory path
- Fixed `LD_LIBRARY_PATH` override to append instead of replacing user's value
- Fixed production log level from `debug` to `info`
- Added build-time warning for unconfigured updater public key

### Maintenance

- 100% Rust codebase (all legacy C++/Qt removed)
- Comprehensive CI: lint, format, test on Linux/Windows/macOS
- Automated release builds with GitHub Actions
