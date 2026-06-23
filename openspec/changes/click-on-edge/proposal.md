## Why

Clicking a shape in the 3D viewer selects the whole shape, but there's no way to pick individual edges. Users who want to apply a fillet or chamfer to specific edges must guess the edge index from `(edge-info shape)` output, then type it manually. This breaks the visual workflow — "see the edge, pick the edge, modify the edge."

## What Changes

- **Edge picking on click**: clicking a rendered edge in the viewer selects that specific edge (visual feedback + event notification)
- **Multi-edge selection**: Shift+click to add, Ctrl+click to toggle, plain click to replace — same modifiers as shape selection
- **Edge selection also selects parent shape**: clicked shape appears in `(selected-shapes)` for visual highlight
- **Edge events in REPL**: selection/deselection events fire through the existing `poll-selection` mechanism, extended with edge information
- **Named shape display**: REPL output shows the `def`-bound variable name (e.g., `edge 3 of b`) instead of opaque `#<rojcad/shape>` pointers
- **New query API**: `(edge-selection)` returns the current edge selection state grouped by shape
- **Convenience helpers**: `(fillet-selected ...)` and `(chamfer-selected ...)` apply operations to selected edges

## Capabilities

### New Capabilities
- `edge-selection`: picking, highlighting, querying, and operating on individual edges of 3D shapes via mouse interaction

### Modified Capabilities

_none_

## Impact

- **src/types.rs**: new atomics for edge selection events; `topo_edge_count` field on `ShapeEntry`/`ShapeData`; `SELECTED_EDGES` global
- **src/viewer/app.rs**: edge picking in `handle_click`; multi-edge selection state; visual highlighting of selected edges
- **src/viewer/pick.rs**: new `pick_edge()` function for screen-space edge hit testing
- **src/main.rs**: new FFI function `rust_poll_edge_selection`; updated selection sync
- **bridge/bridge.c**: extended `_cad_poll_selection_raw` to return edge events (actions 4/5); new `_cad_edge_selection` query
- **boot.janet**: extended `poll-selection` and `poll-viewer` for edge events; `edge-selection` query; `fillet-selected`/`chamfer-selected` helpers; `shape-name` reverse lookup
