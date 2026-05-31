<p align="center">
<img src="./program_info/kcraft-header-black.svg#gh-light-mode-only" alt="KCraft logo" width="50%"/>
<img src="./program_info/kcraft-header.svg#gh-dark-mode-only" alt="KCraft logo" width="50%"/>
</p>

KCraft is a next-generation custom launcher for Minecraft, engineered for extreme performance, absolute predictability, and premium design.

Built entirely with **Rust**, **Tauri**, and **Nuxt/Vue3**, KCraft ditches legacy C++ and Qt implementations to provide a glassmorphism interface, lightning-fast DAG resolving for Modpacks, and zero-compromise autonomy.

# Installation

- All downloads and release binaries for KCraft can be found in our [GitHub Releases](https://github.com/kcraft-org/kcraft/releases).
- KCraft features a **self-updating engine**: download once, and the app will natively upgrade itself whenever a new version is pushed to GitHub.

# Help & Support

KCraft is completely decentralized and independent. There is no official website or Discord server.
Everything happens directly on GitHub:

- Need help? Start a thread in [GitHub Discussions](https://github.com/kcraft-org/kcraft/discussions).
- Found a bug? Open an issue in our [Issue Tracker](https://github.com/kcraft-org/kcraft/issues).

# Development

KCraft is an open-source monument to software engineering. Contributions are welcome directly via Pull Requests.

## Building from Source

To compile KCraft locally, ensure you have Node.js and the Rust toolchain installed:

```bash
# 1. Install frontend dependencies
cd crates/kcraft-gui/frontend
npm ci

# 2. Build the Tauri native application
cd ../src-tauri
cargo build --release
```

## Forking/Redistributing policy

We encourage forking! Just follow the terms of the [GPL-3.0-only license](LICENSE). If you distribute custom builds, please make it clear that your fork is not the official KCraft.

*The logo and related assets are under the CC BY-SA 4.0 license.*
