# kcraft Contributing

## Development Setup

1. Install Rust (stable) via rustup.
2. Clone the repository with `git clone --recursive`.
3. Build: `cargo build --release -p kcraft-gui`.
4. Test: `cargo test --workspace`.

Dependencies (Ubuntu): `libxkbcommon-dev build-essential libssl-dev libfontconfig-dev`.

## CI

All CI checks are defined in `.github/workflows/ci.yml`. Before submitting:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-features`

## Commit Sign-off

All commits must be signed off using `git commit -s`. This certifies compliance with the Developer Certificate of Origin (DCO).
