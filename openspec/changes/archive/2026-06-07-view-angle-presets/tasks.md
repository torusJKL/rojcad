## 1. Rust types — add SetViewAngles variant

- [x] 1.1 Add `SetViewAngles { yaw: f64, pitch: f64, distance: Option<f64> }` variant to `ReplToViewer` enum in `src/types.rs`

## 2. Rust FFI — send commands through channel

- [x] 2.1 Add `rust_view_set_angles(yaw: f64, pitch: f64, has_distance: bool, distance: f64)` extern "C" function in `src/main.rs` that sends `ReplToViewer::SetViewAngles` via the `REPL_TO_VIEWER` OnceLock sender

## 3. Viewer — handle SetViewAngles in render loop

- [x] 3.1 In `src/viewer/app.rs`: add `ReplToViewer::SetViewAngles` match arm to `check_repl_commands` that starts a `FitAnimation` with target pinned to current camera position, animating yaw/pitch/radius to requested values

## 4. C bridge — view-angle JANET_FN

- [x] 4.1 Add `extern void rust_view_set_angles(double yaw, double pitch, int has_distance, double distance);` declaration to `bridge/bridge.c`
- [x] 4.2 Add `cad_view_angle` JANET_FN to `bridge/bridge.c` — accepts 2-3 args (yaw, pitch, optional distance), calls `rust_view_set_angles`
- [x] 4.3 Register `view-angle` in the `cfuns[]` array in `bridge/bridge.c`
- [x] 4.4 Add `{"view-angle", "view"}` entry to `cad_fn_categories[]` in `bridge/bridge.c`

## 5. Janet — named presets in boot.janet

- [x] 5.1 Add `view-front` function with docstring and category metadata in `boot.janet`
- [x] 5.2 Add `view-back` function with docstring and category metadata in `boot.janet`
- [x] 5.3 Add `view-left` function with docstring and category metadata in `boot.janet`
- [x] 5.4 Add `view-right` function with docstring and category metadata in `boot.janet`
- [x] 5.5 Add `view-top` function with docstring and category metadata in `boot.janet`
- [x] 5.6 Add `view-bottom` function with docstring and category metadata in `boot.janet`
- [x] 5.7 Add `view-iso` function with docstring and category metadata in `boot.janet`
- [x] 5.8 Set `:source` and `:category` metadata for all 8 view functions (`view-angle` + 7 presets)

## 6. Build and verify

- [x] 6.1 Run `just check` (fast compile) to find any errors
- [x] 6.2 Run `just build` to produce a debug build
