# KCraft Best Practices & Guidelines

## 1. Project Best Practices (Architecture & Engineering)
- **Zero Placeholder Tolerance**: Never use `unimplemented!()`, `todo!()`, or mock data. Every feature must be built for production from day one.
- **Strict Error Handling**: Use custom Error enums (via `thiserror`) for all operations. `unwrap()` and `expect()` are forbidden outside of test boundaries. All functions must safely return `Result<T, E>`.
- **Fault-Tolerant Concurrency**: Use `tokio` for all IO-bound operations (networking, disk). Shared state must be governed by memory-safe synchronization primitives (`Arc<RwLock<T>>` or `tokio::sync::Mutex`) with semaphore limits for high-bandwidth tasks.
- **Idempotency**: All network downloads and file system writes must be idempotent. Failures must leave the system in a predictable, safe state without corrupted assets.
- **Universal Accessibility**: The application must natively support both fully authenticated Online Mode (Microsoft MSA) and Offline Mode (local profiles). No network restrictions should prevent a user from launching a previously downloaded instance.

## 2. Directory Structure Best Practices
The workspace is split into isolated `crates` following the Principle of Least Privilege and Separation of Concerns:
- `crates/kcraft-core`: Base logic, configurations, shared models, and utility constants.
- `crates/kcraft-net`: Dedicated asynchronous download and network management engine.
- `crates/kcraft-auth`: Universal multimodal authentication handling MSA, Yggdrasil, and Offline generation.
- `crates/kcraft-minecraft`: The logic for mod resolution, dependency SAT checking, JSON manifest parsing, and launch arguments formatting.
- `crates/kcraft-gui`: The Tauri entry point interacting directly with the system window manager.
- `crates/kcraft-gui/frontend`: The Nuxt 3 SPA logic containing pure Vue Composition API logic. No business logic (e.g., launching, authenticating) should exist here; it acts purely as a presentation and state-binding layer.

## 3. Documentation Best Practices
- **Objective Engineering Language**: Documentation must be strictly technical. Avoid all marketing buzzwords (e.g., "premium," "game-changing").
- **Docstrings (`///`)**: Every public struct, enum, and method in the Rust backend must have comprehensive docstrings defining arguments, behavior, and error conditions.
- **Architectural Manifestos**: Large-scale paradigms (like the `kcraft-net` architecture) must be documented in crate-root `lib.rs` and top-level `.md` files.

## 4. Online and Offline Autonomy
- **Online Mode**: Employs OAuth2 Microsoft Authentication, fetching secure XSTS tokens and valid session IDs to communicate with Mojang's session servers.
- **Offline Mode**: Operates entirely within `legacy` local profiles. Network checks for instance launching must immediately short-circuit to local cache verification. If the assets and libraries are fully cached, the game must launch flawlessly without a single network ping.
