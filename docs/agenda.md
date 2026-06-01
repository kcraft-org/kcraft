# kcraft Agenda

## Project Identity

KCraft is a production-ready Minecraft launcher written entirely in Rust. It supports universal authentication (Microsoft MSA, Offline, Ely.by) and cross-platform deployment (Linux, Windows, macOS).

## Architecture

- **GUI**: Slint native UI (no webview, no Node.js). `crates/kcraft-gui/`.
- **Auth**: Microsoft OAuth2 (device code flow), Offline, Ely.by via `kcraft-auth`.
- **Instances**: Minecraft instance management with full launch pipeline via `kcraft-minecraft`.
- **Networking**: Async download engine with resume, validation, and progress via `kcraft-net`.
- **Modpacks**: DAG-based dependency resolver for conflict-free modpack building.

## AI Agent Directives

1. Production-ready code only — no `unimplemented!()`, `todo!()`, or `panic!()`.
2. Before finalizing: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` must all pass.
3. Use professional, objective language. No marketing fluff.
4. All changes must be reflected in CI workflows.
5. All commits must be signed-off.
6. No placeholders, fallbacks, or workarounds in any code path. Every feature must be fully implemented with no stubs or conditional compilation for incomplete features.
7. No criticalities, vulnerabilities, or unsafe code (unless strictly required and documented). Audit every dependency for known security issues.
8. Every error path must be handled with proper user feedback, not silently swallowed.
9. No regressions: verify existing tests pass and add new tests for new functionality.

## Current State (Jun 1, 2026)

Slint migration complete. Old `frontend/` and `src-tauri/` removed. CI/release workflows use only Rust toolchain (no npm/Node.js). Binary at `target/release/kcraft-gui` (36 MB).
