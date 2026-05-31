# KCraft: Roadmap per Eliminazione C++

Questo documento traccia il piano per eliminare tutto il codice C++ (ereditato da PolyMC) e sostituirlo con Rust.

## Obiettivi
- Rimuovere ~13.8KLOC C++ in `launcher/` (no UI) e ~33.9KLOC in `launcher/ui/` (Qt UI)
- Sostituire con crate Rust in `crates/`
- Mantenere le stesse funzionalità (launcher Minecraft multi-platform)

## Sprint Completati

### Sprint 0: Setup CI
- [x] Aggiunto `.github/workflows/rust.yml` per build/test/clippy su linux amd64/arm64
- [x] Aggiornato `build.yml` e `trigger_builds.yml` per includere job Rust
- [x] Rimosso flake.lock, .nix, .ci/ da path-ignore

### Sprint A (0.1): CLI launcher
- [x] Riscritto `crates/kcraft/src/main.rs` come CLI funzionale: `--list`, `--launch <id>`, `--server <host:port>`, `--profile <name>`, `--help`
- [x] Aggiunta dipendenza `kcraft-minecraft` al crate principale
- [x] Fix: `DirectJavaLaunchStep` risolve path librerie relative a data root via `set_lib_dir()`
- [x] Fix: metodi duplicati in `modplatform/mod.rs` e `tokio::fs` → `std::fs` in `download_task.rs`
- [x] Compilazione verificata con toolchain stable

### Sprint B (0.3-0.12): Rimozione librerie C/C++
- [x] Rimosse 7 librerie: `hoedown`, `systeminfo`, `murmur2`, `rainbow`, `LocalPeer`, `gamemode`, `katabasis`
- [x] Rimosse dal CMakeLists.txt principale
- [x] Aggiornato .github/workflows per escludere path delle librerie eliminate

### Sprint 1: Rimozione moduli core C++
- [x] Rimossi file C++ con equivalenti Rust:
  - `GZip.cpp/h` → `flate2` crate
  - `MMCZip.cpp/h` → `zip` crate
  - `Json.cpp/h` → `serde_json`
  - `HoeDown.h` → `pulldown-cmark` (future)
  - `JavaCommon.cpp/h` → `kcraft-java` crate
  - `Filter.cpp/h`, `MMCStrings.cpp/h`, `MMCTime.cpp/h`
  - `FileSystem.cpp/h`, `MessageLevel.cpp/h`, `SkinUtils.cpp/h`
  - `ModDownloadTask.cpp/h`, `DesktopServices.cpp/h`
  - `LoggedProcess.cpp/h`, `KonamiCode.cpp/h`
  - `DefaultVariable.h`, `SeparatorPrefixTree.h`, `RWStorage.h`, `QObjectPtr.h`, `ExponentialSeries.h`
- [x] Rimossa libreria `libraries/javacheck/`
- [x] Rimosse referenze da `launcher/CMakeLists.txt`
- [x] ~3.085 LOC C++ eliminate

### Sprint 2: UI Replacement (Tauri)
- [x] Creata GUI Tauri v2 in `crates/kcraft-gui/`
- [x] Frontend Vite + vanilla JS con lista istanze
- [x] Comandi Tauri: `list_instances`, `launch_instance`
- [x] Build verificata: `cargo build -p kcraft-gui` e `npm run build`
- [x] Compilazione con toolchain stable (1.96.0)
- [x] ~265 pacchetti Rust aggiunti (Tauri + WebKit + GTK)

## Sprint Pianificati

### Sprint 3-5: Porting moduli backend
- [x] Task system: portare `instance_task.rs`, task runner
- [x] Tools & services: upgraders, news, screenshots
- [x] Mod platforms backend: completare download_task, modrinth, curseforge, FTB, ATL

### Sprint 6-10: Porting UI
- [x] UI: dialogs (settings, account, version select, mod download, etc.)
- [x] UI: global pages (instances list, console, news, etc.)
- [x] UI: instance pages (version, mods, resource packs, etc.)
- [x] UI: widgets (version list, mod filter, progress, etc.)
- [x] Rimozione finale della directory `launcher/ui/`

## Sprint Futuri: Verso la Perfezione (Il Miglior Launcher al Mondo)

### Sprint 11: Architettura State-of-the-Art
- [ ] Implementare un sistema di Download Manager asincrono multi-thread perfetto, senza blocking I/O, che satura la banda disponibile usando `tokio` e `reqwest`.
- [ ] Refactor del sistema di Dependency Resolution per mod e pacchetti usando grafi diretti aciclici (DAG) e risolutori SAT per zero conflitti.
- [ ] Validazione ferrea e type-safe per ogni singola struct deserializzata (no panic, no `unwrap` non gestiti).

### Sprint 12: Integrazione ModPlatform Universale
- [ ] Supporto completo e fault-tolerant (senza workaround o placeholder) per l'installazione e l'aggiornamento automatico di mod da CurseForge, Modrinth, FTB, e Technic.
- [ ] Sistema di caching locale globale e deduplicazione delle mod (hardlink) per azzerare lo spreco di disco e scaricare i file una volta sola.

### Sprint 13: UI/UX di Livello Premium (Estrema Perfezione)
- [ ] Riscrittura del frontend Tauri con un design system ultra-reattivo, animazioni a 60+ FPS, design ispirato alle app native più moderne (glassmorphism fluido).
- [ ] Gestione globale dello stato della UI per riflettere all'istante download progressivi e log console in tempo reale tramite bridge WebSocket o server-sent events locali.

### Sprint 14: Zero-Bug Policy e Copertura Test
- [ ] Copertura unitaria >95% per tutta la logica core (crate kcraft-*).
- [ ] Test E2E e di integrazione (launch mock server, finta autenticazione Microsoft).
- [ ] Strict mode per Clippy su tutto l'albero.

## Stato Attuale (2026-05-31)

| Modulo | Stato | Sostituto Rust |
|--------|-------|----------------|
| GZip/MMCZip/Json | ✅ RIMOSSO | flate2, zip, serde_json |
| Core logic (Filter, MMCStrings, etc.) | ✅ RIMOSSO | std + crate workspace |
| Java detection | ✅ RIMOSSO | `kcraft-java` |
| javacheck | ✅ RIMOSSO | `kcraft-java` checker |
| CLI launcher | ✅ RIMOSSO | `kcraft` crate |
| GUI launcher | ✅ COMPLETATO | `kcraft-gui` (Tauri) |
| Librerie C/C++ | ✅ RIMOSSE | varie crate |
| UI (230 file, 33.9KLOC) | ✅ RIMOSSO | Tauri frontend |
| Mod platforms (download, FL/ATL/FTB) | ✅ COMPLETATO | `modplatform/` |
| NBT, QuaZip, TOML++ | ✅ COMPLETATO | Rust equivalenti |
| Auth (MSA/Offline Device Code) | ✅ COMPLETATO | `kcraft-auth` |
