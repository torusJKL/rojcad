## Why

Shapes in the 3D view have hard-to-see edges (dark grey on dark background) and selected shapes lack any edge-level visual feedback — the selection highlight only tints the mesh surface, leaving edges unchanged. This makes it difficult to understand shape topology at a glance.

## What Changes

- Change edge color from dark grey (0.05) to light grey (~0.7) for all unselected shapes
- Change edge color to light blue (~0.4, 0.6, 1.0) for the selected shape
- Add two Janet REPL functions to toggle inactive and active edge visibility independently (`edge-toggle-inactive`, `edge-toggle-active`)
- Add Janet query functions to read current toggle state (`edge-inactive-show?`, `edge-active-show?`)
- Split the monolithic edge vertex buffer into separate active/inactive buffers per frame
- Add two new edge render pipelines (active solid, active dashed) alongside the existing inactive ones
- Replace 1-pixel LineList edge rendering with screen-space instanced quad rendering for controllable thickness
- Add edge thickness control from Janet REPL (`edge-thickness`)
- Add per-channel RGB color control for inactive and active edges (`edge-color-inactive`, `edge-color-active`)
- Render edges after mesh surfaces with depth bias so edges overlay on top of geometry

## Capabilities

### New Capabilities
- `edge-toggle-inactive`: Toggle visibility of edges on non-selected shapes from the Janet REPL
- `edge-toggle-active`: Toggle visibility of edges on the selected shape from the Janet REPL
- `edge-style`: Control edge thickness and per-state colors at runtime from the Janet REPL

### Modified Capabilities
- `3d-viewer`: Edge wireframe changes color from dark grey to light grey. Back-edge toggle (`X` key) continues to work unchanged.
- `viewer-selection`: Selected shape's edges change from "brighter/thicker" (current spec text) to light blue, matching the mesh highlight color.

## Impact

- **Modified files**: `src/viewer/app.rs` (EdgeDrawer pipelines, ViewerState, render logic), `src/viewer/edge.wgsl` (fragment shader colors), `bridge/bridge.c` (new Janet functions), `src/main.rs` (optional — new globals), `src/bridge.rs` (if new extern declarations needed), `src/types.rs` (optional — new atomic globals)
- **No new dependencies**
- **No breaking changes** — existing `X` key toggle for back edges is unaffected
- **No significant performance impact** — one extra edge buffer allocation per frame (already rebuilding per frame)
