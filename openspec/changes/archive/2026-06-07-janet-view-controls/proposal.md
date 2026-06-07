## Why

The 3D viewer toggles hidden (occluded) edge visibility and camera projection mode via keyboard shortcuts (X and O/P), but Janet REPL users have no programmatic control over these settings. This prevents scripting viewer behavior and integrating view changes into automated workflows.

## What Changes

- `show_back_edges` default is flipped from `true` to `false` — hidden edges are off by default
- New Janet functions to get/set/toggle hidden edge visibility
- New Janet functions to get/set/toggle camera projection mode (perspective vs orthographic)
- Keyboard shortcuts (X, O/P) continue to work but now update atomic globals as the single source of truth, same as the Janet functions
- Rust FFI bridge functions added for cross-thread communication
- New `AtomicBool` globals in `types.rs` for lock-free viewer thread reads

## Capabilities

### New Capabilities
- `view-controls`: Janet API for querying and controlling 3D viewer state (hidden edge visibility, camera projection mode)

### Modified Capabilities

*(None — no existing specs are affected)*

## Impact

| File | Change |
|------|--------|
| `src/types.rs` | Add `SHOW_BACK_EDGES` and `PROJECTION_PERSPECTIVE` atomic globals |
| `src/viewer/app.rs` | Remove local `show_back_edges` field; wire keyboard handlers to atomics; sync camera from atomic in `render()` |
| `src/main.rs` | Add 6 FFI `extern "C"` bridge functions (toggle/query/set × 2 features) |
| `bridge/bridge.c` | Add 5 `JANET_FN` blocks and register them |
