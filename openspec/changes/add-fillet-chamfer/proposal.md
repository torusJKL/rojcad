## Why

rojcad currently has `wire-fillet` and `wire-chamfer` for 2D wires, but cannot apply these operations to 3D solids/shells. Users need to round edges and bevel corners on solid models — a fundamental CAD capability. opencascade-rs already provides the underlying `Shape::fillet_edges()` and `Shape::chamfer_edges()` APIs; this change exposes them to the Janet DSL.

## What Changes

- Add `fillet` and `chamfer` functions for 3D solids/shells in the Janet DSL
- Support optional edge selection via `:e` keyword (tuple of indices)
- Add `edge-info` function to enumerate edges with index/type/position for discovery
- Add `highlight-edge` and `highlight-edge-clear` to visually preview edge selections in the viewer
- Add the `spork` JSON module (`json.c`) to the vendored Janet build for `json/decode` and `json/encode`
- Replace the custom `json-encode`/`json-escape` in `boot.janet` with spork's native `json/encode`

## Capabilities

### New Capabilities
- `solid-fillet-chamfer`: Fillet and chamfer operations on 3D solids/shells with optional edge selection via `:e` keyword
- `edge-queries`: Edge enumeration and metadata querying (`edge-info`) to discover edge indices for selection
- `edge-highlight`: Visual edge highlighting in the viewer to preview selections before operations

### Modified Capabilities

None. The existing `wire-operations` capability (wire-fillet/wire-chamfer) is unchanged.

## Impact

- **New files**: `vendor/core/spork_json.c` (copied from spork repository, entry point renamed)
- **Modified files**:
  - `src/cad.rs` — new `shape_fillet()`, `shape_chamfer()`, `edge_info()` functions
  - `src/main.rs` — new FFI bridge functions + spork JSON registration
  - `src/bridge.rs` — spork JSON binding declaration
  - `bridge/bridge.c` — new JANET_FN wrappers for fillet/chamfer/edge-info/highlight-edge
  - `boot.janet` — new Janet wrappers + spork JSON replaces custom json-encode
  - `boot/model.janet` — add fillet/chamfer to cad-shape-fns
  - `src/types.rs`, `src/viewer/` — edge highlight viewer state
- **Dependencies**: `serde_json` added to `Cargo.toml` (for serializing EdgeInfo in Rust)
