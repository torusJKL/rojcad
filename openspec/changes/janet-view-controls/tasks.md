## 1. Rust globals and viewer wiring

- [x] 1.1 Add `SHOW_BACK_EDGES: AtomicBool::new(false)` and `PROJECTION_PERSPECTIVE: AtomicBool::new(true)` to `src/types.rs`
- [x] 1.2 In `src/viewer/app.rs`: remove `show_back_edges` field from `ViewerState` struct and its initializer
- [x] 1.3 In `src/viewer/app.rs`: change X key handler to `SHOW_BACK_EDGES.fetch_xor(true, ...)` and O/P handler to `PROJECTION_PERSPECTIVE.fetch_xor(true, ...)`
- [x] 1.4 In `src/viewer/app.rs`: at top of `render()`, sync `state.camera.perspective` from `PROJECTION_PERSPECTIVE` atomic
- [x] 1.5 In `src/viewer/app.rs`: replace `state.show_back_edges` usage with `SHOW_BACK_EDGES.load()` in edge render call
- [x] 1.6 Add `SHOW_BACK_EDGES` and `PROJECTION_PERSPECTIVE` imports to `src/viewer/app.rs`

## 2. FFI bridge functions

- [x] 2.1 In `src/main.rs`: add 6 `extern "C"` functions — `rust_back_edges_toggle`, `rust_back_edges_showing`, `rust_back_edges_set`, `rust_projection_perspective_toggle`, `rust_projection_perspective_showing`, `rust_projection_perspective_set`
- [x] 2.2 Add `SHOW_BACK_EDGES` and `PROJECTION_PERSPECTIVE` imports to `src/main.rs`

## 3. Janet functions in C bridge

- [x] 3.1 In `bridge/bridge.c`: add `cad_edge_hidden_toggle` JANET_FN — 0 args, toggle, return bool
- [x] 3.2 In `bridge/bridge.c`: add `cad_edge_hidden_showing` JANET_FN — 0 args, query, return bool
- [x] 3.3 In `bridge/bridge.c`: add `cad_edge_hidden` JANET_FN — 0-1 args, no arg = query, 1 arg = set
- [x] 3.4 In `bridge/bridge.c`: add `cad_projection_toggle` JANET_FN — 0 args, toggle, return bool
- [x] 3.5 In `bridge/bridge.c`: add `cad_projection_perspective` JANET_FN — 0-1 args, no arg = query, 1 arg = set
- [x] 3.6 Register all 5 functions in `cad_register_functions()` inside `bridge/bridge.c`

## 4. Build and verify

- [x] 4.1 Run `just check` (fast compile check) to find any errors
- [x] 4.2 Run `just build` to produce a debug build
- [ ] 4.3 Verify keyboard shortcuts (X, O/P) still work when viewer is running (manual — needs display)
