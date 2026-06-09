## Why

Middle-mouse-button panning is unreliable on Wayland (horizontal MMB motion can trigger zoom due to compositor/wheel interactions). Shift+LMB is a standard convention in 3D tools (Blender, many CAD apps) and avoids platform-specific input issues entirely.

## What Changes

- **Shift+LMB drag** performs camera pan (replaces MMB as the primary pan gesture)
- **MMB drag** is kept as an alternative pan method for users who prefer it
- **LMB drag** (without Shift) continues to orbit/rotate the camera
- **LMB click** (press + release without significant movement) continues to handle shape selection
- **Shift+LMB click** (press + release without significant movement) continues to add to selection
- The click-vs-drag distinction uses the existing 3px threshold on release — no selection fires if the user dragged

## Capabilities

### New Capabilities
- `camera-pan`: Camera panning controls — Shift+LMB drag as primary, MMB drag as fallback

### Modified Capabilities
- `viewer-selection`: The "Drag does not trigger selection" scenario currently states "the camera orbits during the drag" — needs updating to account for Shift+LMB panning during drag
- `view-controls`: Existing spec covers edge visibility and projection only, no mouse controls — will add a reference to the new `camera-pan` capability

## Impact

- `src/viewer/app.rs:1241-1253` — CursorMoved handler: add Shift modifier check to LMB drag to route to pan vs rotate
- No API changes — Janet functions and viewer selection remain unchanged
- No new dependencies
