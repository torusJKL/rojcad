## Why

Zoom (scaling radius toward target) and dolly (translating the whole camera rig forward/backward) are distinct camera motions with different feel and use cases. Dolly preserves perspective while moving through the scene — essential for walkthroughs, inspecting details, and positioning before orbit. Without it, users can only zoom, which changes the orbit center relationship and feels like a lens adjustment, not movement through space.

## What Changes

- Add `dolly()` method to `OrbitCamera` that translates `target` along the view direction
- Wire `Shift+scroll` → dolly in/out
- Wire `Shift+RMB drag` → dolly in/out (replacing the RMB=zoom behavior when Shift is held)
- Add new shortcut entries to help overlay
- No breaking changes — Zoom still works via unmodified scroll or RMB drag

## Capabilities

### New Capabilities
- `camera-dolly`: Forward/backward camera translation (dolly) through the scene, distinct from zoom

### Modified Capabilities
*(none)*

## Impact

- `src/viewer/camera.rs` — add `dolly()` method (~5 lines)
- `src/viewer/app.rs` — two modifier checks (~10 lines changed in input handlers)
- `src/viewer/help.rs` — add Shift+Scroll and Shift+RMB entries
