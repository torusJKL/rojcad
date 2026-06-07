## Why

The 3D viewer's orbit camera has a fixed starting position (radius=50, origin target), and users must manually pan/zoom/orbit to frame shapes of interest. No programmatic way exists to frame the camera on one or more shapes, which makes inspection workflows tedious and prevents scripting camera behavior.

## What Changes

- New `(view-fit & shapes ; :keep-angle)` Janet function — frames camera on the union bounding box of explicitly provided shapes
- New `(view-fit-all ; :keep-angle)` Janet function — frames camera on the bounding box of all visible shapes; resets to default camera if no shapes visible
- Both functions animate the camera over 0.5s with ease-in-out interpolation
- `:keep-angle` keyword preserves the current yaw/pitch instead of resetting to default isometric (0, 0.4)
- `(view-fit)` with zero shapes panics with an error message
- mpsc channel added for REPL→Viewer communication (extensible for future commands)

## Capabilities

### New Capabilities
- `view-fit`: Janet API for fitting the 3D camera to shape bounding boxes (explicit shapes and all visible)

### Modified Capabilities

*(None — no existing specs are affected)*

## Impact

| File | Change |
|------|--------|
| `src/types.rs` | Add `ReplToViewer` enum with `FitToBounds` variant |
| `src/viewer/mod.rs` | `spawn_viewer()` accepts `Receiver<ReplToViewer>` |
| `src/viewer/app.rs` | Add `FitAnimation` struct, `fit_to_bounds()` method, channel polling in `render()` |
| `src/main.rs` | Add `REPL_TO_VIEWER` global sender, 2 `extern "C"` bridge functions, wire channel at viewer startup |
| `bridge/bridge.c` | Add 2 `JANET_FN` blocks (cad_view_fit, cad_view_fit_all) and register them |
