## Why

The 3D viewer currently supports single-shape selection only. Users working with complex assemblies or performing comparisons need to select multiple shapes simultaneously — for example, to inspect multiple parts, apply batch operations, or visualize relationships. Adding multi-select with keyboard modifiers brings the viewer in line with standard 3D application conventions.

## What Changes

- **Multi-select state**: Viewer selection internally changes from `Option<ShapeId>` to `HashSet<ShapeId>`
- **Modifier-based selection modes**:
  - **Plain click**: Replace selection with clicked shape (existing behavior, but now fires on mouse release instead of press)
  - **Shift+click**: Add clicked shape to selection (no removal)
  - **Ctrl+click**: Toggle clicked shape in/out of selection
- **Wait-for-release**: All selection events trigger on mouse button release, not press, to avoid accidental selection during drag-to-orbit
- **Click-on-nothing behavior**: Plain click on empty space clears selection; Ctrl/Shift+click on empty space is a no-op
- **Janet bridge**: `poll-selection` updated to report toggle-off and multi-select events
- **Rendering**: Highlight and edge coloring applies to all selected shapes uniformly

## Capabilities

### New Capabilities
- `viewer-selection`: Mouse and keyboard-driven shape selection in the 3D viewer, including multi-select via modifiers

### Modified Capabilities

*None.*

## Impact

- **src/viewer/app.rs**: ViewerState, handle_click, SurfaceDrawer::render, edge-instance building
- **src/types.rs**: Selection atomics (`LAST_SELECTION` → enhanced with `LAST_SELECTION_ACTION`)
- **src/viewer/mod.rs**: `ViewerToRepl` enum updated
- **src/main.rs**: `rust_poll_selection` updated to return action+id pair
- **bridge/bridge.c**: `cad_poll_selection` handles toggle-off events; fixes existing `:deselected` keyword return bug
- **boot.janet**: Polling loop updated for richer event reporting
