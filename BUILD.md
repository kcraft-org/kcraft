# Build Instructions

KCraft is built with Rust and Node.js.

## Prerequisites

1. Node.js (v20+)
2. Rust Toolchain (stable)
3. Platform-specific build tools for Tauri (e.g. `libwebkit2gtk-4.1-dev` and `build-essential` on Linux)

## Compiling

```bash
# Clone the repository
git clone https://github.com/kcraft-org/kcraft.git
cd kcraft

# Build the frontend
cd crates/kcraft-gui/frontend
npm ci
npm run generate

# Build the Tauri application
cd ../src-tauri
cargo build --release
```

The compiled binary will be located in `target/release/`.
