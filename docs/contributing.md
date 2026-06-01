# kcraft Contributing

## Development Setup

Install Rust (stable) via rustup. Clone the repository with `git clone --recursive`. Build with `cargo build --release -p kcraft-gui`. Test with `cargo test --workspace`. Dependencies (Ubuntu): `libxkbcommon-dev build-essential libssl-dev libfontconfig-dev`.

## CI

All CI checks are defined in `.github/workflows/ci.yml`. Before submitting run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --all-features`.

## Commit Sign-off

All commits must be signed off using `git commit -s`. This certifies compliance with the Developer Certificate of Origin (DCO).
