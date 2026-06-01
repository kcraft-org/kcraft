# kcraft Build

## Prerequisites

- Rust stable toolchain
- libxkbcommon-dev, libssl-dev, libfontconfig-dev (Linux)

## Build Commands

```shell
cargo build --release -p kcraft-gui
```

The binary is produced at `target/release/kcraft-gui`.

## Cross-Platform

Kraft builds natively on Linux, Windows, and macOS. Each CI build targets the native platform. macOS builds are produced for both x86_64 and aarch64.
