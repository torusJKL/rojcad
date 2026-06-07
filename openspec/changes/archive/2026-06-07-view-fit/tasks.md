## 1. Types and Communication Channel

- [x] 1.1 Add `ReplToViewer` enum to `src/types.rs` with `FitToBounds` variant (import `DVec3` from `glam`)
- [x] 1.2 Fix `src/viewer/mod.rs` — add `use std::sync::mpsc::Receiver`, change `spawn_viewer()` signature to accept `Receiver<ReplToViewer>`, pass it to `run_viewer()`
- [x] 1.3 Add `REPL_TO_VIEWER: OnceLock<mpsc::Sender<ReplToViewer>>` global to `src/main.rs`
- [x] 1.4 Create reverse channel in `main()` before `spawn_viewer()` call, store sender in global, pass receiver to `spawn_viewer()`

## 2. Fit Animation on Viewer Thread

- [x] 2.1 Add `fit_animation: Option<FitAnimation>` field to `ViewerState` in `src/viewer/app.rs`
- [x] 2.2 Add `repl_rx: Receiver<ReplToViewer>` field to `ViewerApp` in `src/viewer/app.rs`
- [x] 2.3 Create `FitAnimation` struct with start/end target, radius, yaw, pitch, elapsed, and duration (0.5s), using existing `ease_in_out()` function
- [x] 2.4 Implement `FitAnimation::update(&mut self, camera: &mut OrbitCamera, dt: f64) -> bool` — lerp all four parameters, return `true` when complete
- [x] 2.5 Implement `check_repl_commands(state, rx)` function that polls `try_recv()` and calls `state.fit_to_bounds()` for `FitToBounds` messages
- [x] 2.6 Implement `ViewerState::fit_to_bounds(center, radius, keep_angle)` — compute target radius based on perspective/orthographic mode, set `fit_animation` with start/end states
- [x] 2.7 Wire `check_repl_commands()` at the top of `render()`, and update `fit_animation` alongside existing `animation.update()` in render loop

## 3. Rust FFI Bridge Functions

- [x] 3.1 Add `use types::ReplToViewer` and `use std::sync::mpsc` to imports in `src/main.rs`
- [x] 3.2 Implement `compute_union_bounds` in `src/cad.rs` (called from main via `cad::compute_union_bounds`)
- [x] 3.3 Implement `rust_view_fit_shapes(shapes: *mut *mut c_void, count: i32, keep_angle: bool)` — cast pointers to `&ShapeData`, compute bounds, send `FitToBounds` via `REPL_TO_VIEWER`
- [x] 3.4 Implement `rust_view_fit_all(include_hidden: bool, keep_angle: bool)` — get visible (or all if `include_hidden`) shapes from `global_shape_registry()`, compute bounds, send `FitToBounds`. If no shapes found, send reset with `center=origin, radius=50, keep_angle=false`

## 4. C Bridge (bridge/bridge.c)

- [x] 4.1 Add `extern` forward declarations for `rust_view_fit_shapes` and `rust_view_fit_all` (signature updated to `int include_hidden, int keep_angle`)
- [x] 4.2 Implement `cad_view_fit` JANET_FN — parse non-keyword args as shapes (require ≥ 1), detect `:reset` keyword, build `void*` array, call `rust_view_fit_shapes`
- [x] 4.3 Implement `cad_view_fit_all` JANET_FN — detect optional `:hidden` and `:reset` keywords, call `rust_view_fit_all`
- [x] 4.4 Register both functions in `JanetReg cfuns[]` array
- [x] 4.5 Add category metadata entries: `"view-fit"` → `"view"`, `"view-fit-all"` → `"view"`
- [x] 4.6 Add `:hidden` keyword parsing to `cad_view_fit_all`
- [x] 4.7 Update docstrings to document `:reset` instead of `:keep-angle`

## 5. Registry and FFI Updates for :hidden

- [x] 5.1 Add `all_shapes()` method to `ShapeRegistry` in `src/types.rs` (returns all entries regardless of visibility)
- [x] 5.2 Update `rust_view_fit_all` signature to `(include_hidden: bool, keep_angle: bool)` in `src/main.rs`
- [x] 5.3 Update extern declaration in `bridge/bridge.c` to match new signature
- [x] 5.4 Replace `:keep-angle` with `:reset` in `cad_view_fit` and `cad_view_fit_all` JANET_FNs — default is now keep-angle, `:reset` explicitly resets
- [x] 5.5 Invert `keep_angle` boolean in FFI boundary: `reset=false` → `keep_angle=true` (default keep), `reset=true` → `keep_angle=false` (reset)

## 6. Tests

- [x] 5.1 Add `test_fit_bounds_single` — feed known vertex data to compute_union_bounds, verify center and radius
- [x] 5.2 Add `test_fit_bounds_multiple` — two offset vertex sets, verify union AABB
- [x] 5.3 Add `test_fit_bounds_no_mesh` — shape with `mesh: None` returns `None`
- [x] 5.4 Add `test_fit_bounds_single_vertex` — degenerate case, center at vertex, radius ~0

## 7. Build and Verify

- [x] 7.1 Run `just fmt` and `just check` to verify compilation
- [x] 7.2 Run unit tests
- [x] 7.3 Manual smoke test: launch viewer, create a shape, call `(view-fit my-shape)`, verify camera animates to frame it
- [x] 7.4 Manual test: `(view-fit)`, `(view-fit :reset ...)`, `(view-fit-all)`, `(view-fit-all :hidden :reset)`, `(view-fit)` no args (expect panic)
