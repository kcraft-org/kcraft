# kcraft Agenda

## Project Identity

KCraft is a production-ready Minecraft launcher written entirely in Rust. It supports universal authentication (Microsoft MSA, Offline, Ely.by) and cross-platform deployment (Linux, Windows, macOS).

## Architecture

The GUI is built with Slint (native UI, no webview, no Node.js) in `crates/gui/`. Authentication uses Microsoft OAuth2 (device code flow), offline mode, and Ely.by via `kcraft-auth`. Instance management with full launch pipeline is in `kcraft-minecraft`. Networking provides an async download engine with resume, validation, and progress in `kcraft-net`. Modpack building includes a DAG-based dependency resolver for conflict-free resolution.

## AI Agent Directives

Production-ready code only — no `unimplemented!()`, `todo!()`, or `panic!()`. Before finalizing every change, run `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` — all must pass. Use professional, objective language with no marketing fluff. All changes must be reflected in CI workflows and all commits must be signed-off. No placeholders, fallbacks, or workarounds in any code path — every feature must be fully implemented with no stubs or conditional compilation for incomplete features. No criticalities, vulnerabilities, or unsafe code (unless strictly required and documented). Audit every dependency for known security issues. Every error path must be handled with proper user feedback, not silently swallowed. No regressions: verify existing tests pass and add new tests for new functionality.

## Current State (Jun 1, 2026)

Slint migration complete. Old `frontend/` and `src-tauri/` removed. CI/release workflows use only Rust toolchain (no npm/Node.js). Binary at `target/release/kcraft-gui` (36 MB).
