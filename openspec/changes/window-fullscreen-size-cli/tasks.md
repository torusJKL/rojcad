## 1. Shared state — atomics and ReplToViewer variants

- [x] 1.1 Add `WINDOW_WIDTH`, `WINDOW_HEIGHT`, `WINDOW_FULLSCREEN`, `WINDOW_MAXIMIZED` atomics in `src/types.rs`
- [x] 1.2 Add `SetWindowSize`, `SetFullscreen`, `SetMaximized` variants to `ReplToViewer` enum in `src/types.rs`

## 2. ViewerConfig plumbing

- [x] 2.1 Add `ViewerConfig` struct (width, height, maximized) to `src/viewer/mod.rs`
- [x] 2.2 Update `spawn_viewer()` to accept `ViewerConfig` parameter
- [x] 2.3 Thread config through `run_viewer()` into `ViewerApp` struct in `src/viewer/app.rs`

## 3. CLI argument parsing

- [x] 3.1 Add `parse_size_args()` function in `src/main.rs` for `--width`/`--height`
- [x] 3.2 Compute maximized flag from CLI args
- [x] 3.3 Create `ViewerConfig` and pass to `spawn_viewer()`

## 4. Viewer window initialization

- [x] 4.1 Use `config.width` / `config.height` instead of hardcoded values in `resumed()`
- [x] 4.2 Apply `with_maximized(true)` on window attributes if `config.maximized`
- [x] 4.3 Initialize atomics after window creation

## 5. Viewer command handling

- [x] 5.1 Handle `SetWindowSize` in `check_repl_commands()`
- [x] 5.2 Handle `SetFullscreen` in `check_repl_commands()`
- [x] 5.3 Handle `SetMaximized` in `check_repl_commands()`
- [x] 5.4 Update `WINDOW_WIDTH` / `WINDOW_HEIGHT` atomics on `Resized` event

## 6. Janet bridge — Rust extern C functions

- [x] 6.1 Add `rust_window_set_size()` / `rust_window_size_query()` in `src/main.rs`
- [x] 6.2 Add `rust_window_set_fullscreen()` / `rust_window_fullscreen_query()` in `src/main.rs`
- [x] 6.3 Add `rust_window_set_maximized()` / `rust_window_maximized_query()` in `src/main.rs`

## 7. Janet bridge — C wrappers

- [x] 7.1 Add `JANET_FN(cad_window_size)` / `cad_window_size_query` in `bridge.c`
- [x] 7.2 Add `JANET_FN(cad_window_fullscreen)` / `cad_window_fullscreen_query` in `bridge.c`
- [x] 7.3 Add `JANET_FN(cad_window_maximized)` / `cad_window_maximized_query` in `bridge.c`
- [x] 7.4 Register all 6 functions in `cad_register_functions` table + category entries in `bridge.c`

## 8. Help overlay update

- [x] 8.1 Add `--width <PX>` and `--height <PX>` entries to CLI args grid in `src/viewer/help.rs`

## 9. Verify build

- [x] 9.1 Run `just check` to verify compilation
- [x] 9.2 Run `just test` to ensure no test regressions
- [x] 9.3 Run `just lint` to verify clippy compliance
