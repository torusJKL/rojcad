## Why

Janet scripts currently have no way to enumerate registered shapes or query selection state. A user who wants to "hide all selected shapes" or "print the types of every visible shape" must manually track shape references in Janet variables — there's no way to query the viewer or registry programmatically. This limits script automation and REPL interactivity.

## What Changes

- **`(selected-shapes)`**: New Janet function returning a tuple of `ShapeData` abstract values that are currently selected in the 3D viewer
- **`(list-shapes &keys :visible :hidden)`**: New Janet function returning a tuple of all registered `ShapeData` abstract values, optionally filtered by visibility state
- **Shape pointer map**: A global `HashMap<ShapeId, *mut c_void>` mapping each registered shape's ID to its Janet GC-allocated `ShapeData` pointer, populated at shape creation time and cleaned up on `Drop`
- **Selection state sync**: The viewer thread writes the current selection set to a shared global (`RwLock<HashSet<ShapeId>>`) alongside the existing atomic event reporting, so `selected-shapes` can read it
- **Registry extension**: Add `hidden_shapes()` method to `ShapeRegistry` for the `:hidden` filter
- **C bridge additions**: Two new `JANET_FN` entries (`cad_selected_shapes`, `cad_list_shapes`) plus a `rust_register_abstract()` call in every shape-creating JANET_FN
- **Rust FFI**: New `extern "C"` functions for querying selection IDs, registered shape IDs, and looking up `ShapeData*` by ID

## Capabilities

### New Capabilities
- `shape-queries`: Programmatic querying of shape state from Janet — getting currently selected shapes, and enumerating all registered shapes with optional visibility filtering. Includes the shape pointer map infrastructure needed to return real `ShapeData` objects (not just IDs).

### Modified Capabilities

*None.*

## Impact

- **bridge/bridge.c**: Every shape-creating `JANET_FN` (~25 functions) needs one added line (`rust_register_abstract(shape)`) plus two new `JANET_FN` blocks and registration entries
- **src/types.rs**: New `SHAPE_PTR_MAP` global, `SELECTED_IDS` global, `register_shape_pointer()` / `unregister_shape_pointer()` functions; `ShapeData::drop` extended; `ShapeRegistry::hidden_shapes()` method
- **src/main.rs**: New `extern "C"` FFI functions for pointer lookup, ID queries, and memory deallocation
- **src/viewer/app.rs**: Viewer writes `SELECTED_IDS` global after every selection change in `handle_click`
