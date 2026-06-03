# Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for the project roadmap.

## Sprint: Feature Parity & Quality (v1.1.0)

After auditing the launcher landscape and identifying gaps, we target the following priorities:

### P0 — Ship-blocking
- [ ] Auto Java management: per-instance Java auto-detection and auto-download
- [ ] Mod auto-update from CurseForge and Modrinth
- [ ] Instance cloning (copy full instance with all config)
- [ ] Crash reporter with structured log capture and stack trace
- [ ] Built-in self-updater (binary replacement on launch)

### P1 — Competitive parity
- [ ] World management panel with restore from backup
- [ ] Resource pack browser and one-click install
- [ ] Shader pack installation from supported platforms
- [ ] Per-instance JVM args UI (memory, GC, custom flags)
- [ ] Portable mode (all data relative to binary path)
- [ ] Custom theme support via Slint theming API

### P2 — Advantage features
- [ ] Pre-launch and post-launch shell script hooks per instance
- [ ] Quilt mod loader support
- [ ] FTB and Technic modpack import
- [ ] Server pack export from client instance
- [ ] Cloud sync / instance sharing via URL

### P3 — Aspirational
- [ ] PvP overlay (FPS, CPS, coordinates, keystrokes)
- [ ] Controller support via SDL2 binding
- [ ] Mobile build target (Android/iOS via Slint)
- [ ] Plugin/extension API for community modules
- [ ] Instance marketplace / modpack browser inside launcher
