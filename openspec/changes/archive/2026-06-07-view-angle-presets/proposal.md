## Why

The 3D viewer can only be repositioned via keyboard shortcuts (Ctrl+1/3/7 for axis-aligned views, mouse for orbit) within the viewer window. There is no programmatic way to set camera angles from the Janet REPL, preventing scripting of view changes, reproducible camera setups, and headless-first workflows.

## What Changes

- New `ReplToViewer::SetViewAngles { yaw, pitch, distance }` variant on the existing REPL→Viewer mpsc channel
- New `view-angle` C JANET_FN in `bridge/bridge.c` — generic function accepting yaw, pitch, and optional distance
- Seven named view-preset functions defined in `boot.janet` as wrappers around `view-angle`:
  - `view-front`, `view-back`, `view-left`, `view-right`, `view-top`, `view-bottom`, `view-iso`
- New `rust_view_set_angles` Rust FFI function in `src/main.rs` to send commands through the mpsc channel
- Viewer thread handles `SetViewAngles` via the existing `FitAnimation` system (smooth 0.5s ease-in-out)
- All presets accept an optional distance argument to set zoom level
- Named preset docstrings document their yaw/pitch values

## Capabilities

### New Capabilities
- `camera-view`: Programmatic camera angle control from the Janet REPL with named presets and arbitrary yaw/pitch/distance

### Modified Capabilities

*(None — no existing specs are affected)*

## Impact

| File | Change |
|------|--------|
| `src/types.rs` | Add `SetViewAngles` variant to `ReplToViewer` enum |
| `src/viewer/app.rs` | Handle `SetViewAngles` in `check_repl_commands` via `FitAnimation` |
| `src/main.rs` | Add `rust_view_set_angles` extern "C" FFI function (~10 lines) |
| `bridge/bridge.c` | Add extern declaration + `cad_view_angle` JANET_FN + one registration entry + one category entry |
| `boot.janet` | Add 7 named preset functions with docstrings and category metadata |
