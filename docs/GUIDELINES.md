# kcraft Guidelines

## Zero Placeholder Tolerance

Never use `unimplemented!()`, `todo!()`, or mock data. Every feature must be production-ready from day one. Use custom Error enums (via `thiserror`) for all operations. `unwrap()` and `expect()` are forbidden outside of test boundaries.

## Fault-Tolerant Concurrency

Use `tokio` for all IO-bound operations (networking, disk). Shared state must be governed by memory-safe synchronization primitives (`Arc<RwLock<T>>` or `tokio::sync::Mutex`) with semaphore limits for high-bandwidth tasks. All network downloads and filesystem writes must be idempotent.

## Code Quality

`cargo clippy --workspace --all-targets -- -D warnings` must pass before any merge. `cargo fmt --all -- --check` must pass. All tests must pass via `cargo test --workspace`.

## Documentation

Use objective engineering language — no marketing buzzwords. Every public struct, enum, and method must have comprehensive docstrings defining arguments, behavior, and error conditions.

## Security

All commits must be signed-off (DCO). Secrets and keys must never be committed to the repository.
