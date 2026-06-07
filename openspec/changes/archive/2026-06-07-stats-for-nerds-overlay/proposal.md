## Why

rojcad's 3D viewer currently has zero HUD elements — no camera info, shape counts, or performance feedback. Users orbit, zoom, toggle edges, and watch shapes appear, but have no quantitative sense of what's happening. A stats overlay gives real-time feedback on camera state, scene complexity, rendering performance, and toggle status, making the viewer feel like a professional CAD tool rather than a black box.

## What Changes

- Add egui (immediate-mode GUI library) as a dependency for rendering the overlay
- Display camera state: yaw, pitch, zoom/radius, projection mode, and a human-readable view name (Front, Right, Top, Iso, Custom, etc.)
- Display shape statistics: total shapes, visible shapes, hidden shapes, selected shape ID
- Display geometry statistics: total triangle count, total vertex count
- Display toggle states: back-edge visibility (X key), overlay visibility
- Display performance: FPS and frame time (smoothed via ring buffer)
- Overlay renders as a draggable egui floating window
- Add keyboard shortcut `Ctrl + Shift + Alt + S` to toggle the overlay
- Add Janet function `stats-overlay` (toggle, get, set) following the same pattern as `edge-hidden` and `projection-perspective`
- Add look-at preset detection that maps current (yaw, pitch) to a named view or "Custom"
- New dependencies: `egui`, `egui-wgpu`, `egui-winit` for GPU-accelerated UI rendering

## Capabilities

### New Capabilities
- `viewer-stats-overlay`: Real-time HUD showing camera, scene, and performance statistics in the 3D viewport

### Modified Capabilities

- *None — this is purely additive; no existing capability changes its requirements.*

## Impact

- **New dependencies**: `egui`, `egui-wgpu`, `egui-winit`
- **Removed dependencies**: `ab_glyph`
- **New module**: `src/viewer/stats.rs` (~150 lines) containing egui UI construction and stat collection helpers
- **Modified files**: `Cargo.toml` (swap deps), `src/types.rs` (new atomic toggle), `src/viewer/mod.rs` (pub mod), `src/viewer/app.rs` (integrate egui context + renderer into ViewerState, feed events, render overlay), `src/main.rs` (Rust bridge functions), `bridge/bridge.c` (Janet function registration)
- **No breaking changes**: toggle defaults to on; can be hidden at any time
