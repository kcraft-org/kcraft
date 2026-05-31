# KCraft: Agentic Manifesto & Software Blueprint

## 1. Vision and Identity
KCraft is a state-of-the-art, production-ready Minecraft launcher completely written in Rust. 
It is not a prototype, marketing demo, or a "premium" toy. It is serious, system-level software engineered for fault tolerance, memory safety, and uncompromising performance.

The fundamental objective of this project is to create the **best Minecraft launcher in the world**. It supports universal multimodal authentication (Online Microsoft Account Flow and Offline Play) guaranteeing universal access.

## 2. Core Architectural Pillars
- **Memory Safety & Performance**: The transition to Rust strictly enforces the `cargo clippy --workspace --all-targets -- -D warnings` standard. There are zero unhandled panics, no default bypasses, and no redundant logic.
- **Asynchronous Concurrency**: Heavy usage of `tokio` and `reqwest` provides a highly parallelized backend capable of managing multiple file downloads (assets, libraries, mods, indices) concurrently without blocking the main execution thread.
- **Robust Mod Platform Integration**: KCraft interfaces natively with CurseForge, Modrinth, FTB, and ATL. Integrations must be perfect, meaning robust error handling without API mocks, fallback placeholders, or "todo" macros.
- **Lightweight, Responsive UI**: The frontend utilizes Tauri and Nuxt 3 with Vite. It features a robust, modern aesthetic (Lucide icons, CSS variables, Vue Composition API) focused on responsiveness and zero latency. It operates as a true desktop application with native system integration.

## 3. Development Directives for the AI Agent
When operating on this repository, you must adhere to the following directives:

1. **Production-Ready Code Only**: Do not use `unimplemented!()`, `todo!()`, or `panic!()`. Every edge case must return a `Result` or `Option` and be handled gracefully. Do not write placeholder mock data.
2. **Zero-Bug Tolerance**: Before finalizing any step, `cargo test` and `cargo clippy` must pass without a single warning.
3. **No Marketing Fluff**: We are writing engineering software. Use professional, objective terminology. "Premium", "marketing", and "hype" have no place in commit messages, documentation, or the codebase.
4. **Deterministic and Idempotent Design**: State mutations, file system operations, and network requests should be fault-tolerant. If a download fails, it should cleanly abort and reset its state without leaving corrupted artifacts.
5. **Continuous Integration**: Ensure that all changes are reflected in CI (`.github/workflows`). The build matrices must pass across all platforms natively.

## 4. Current Roadmap
With the legacy infrastructure fully eradicated, the architecture is 100% Rust and NuxtJS. The immediate objective is to advance into the perfection roadmap:

- **UI/UX Realism**: Achieve 60+ FPS responsive UI syncing real-time download events from the Rust backend via Tauri IPC events to the Nuxt frontend.
- **Zero-Bug Validation**: Ensure >95% code coverage for `kcraft-*` crates, plus complete E2E integration test suites.
- **Modpack Engineering**: Implement a complex, Vue-based Drag-and-Drop Modpack builder in the Nuxt UI, interacting natively with the Rust SAT resolver for zero-conflict dependency mapping.
- **Telemetry & Lifecycle**: Integrate intelligent application lifecycle management, self-updating mechanisms, and robust error tracing using `tracing` and structured logging over Tauri.
- **Offline Autonomy**: Guarantee flawless fallback execution in disconnected environments, ensuring previously cached assets seamlessly boot instances without network timeout latency.
- **Final Polishing**: Achieve ultimate architectural stability. Optimize memory allocations, streamline binary size via advanced Cargo profiles, and certify the software for mass distribution.

## 5. Conclusion
KCraft represents the pinnacle of modern software engineering applied to game launchers. The AI agent managing this repository operates as a senior systems engineer tasked with reaching absolute, uncompromised perfection.
