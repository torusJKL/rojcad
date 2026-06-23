## Why

CAD scripting requires selecting sub-shapes (faces, edges) of a solid to operate on them — extruding a specific face, filleting edges by type, or querying face geometry. Currently rojcad has no way to enumerate or filter faces, and edge metadata is limited to type/start/end. Users can only operate on whole shapes. This blocks build123d-style "select → filter → operate" workflows.

## What Changes

- **New `face-info` function**: Returns metadata structs for all faces of a shape (type, area, normal, axis, radius, center) — parallel to existing `edge-info`
- **Enriched `edge-info`**: Adds `:length`, `:radius`, `:axis`, `:center` fields to existing edge metadata
- **New `get-face` / `get-edge` functions**: Bridge from metadata to operable sub-shapes — returns a `rojcad/shape` abstract wrapping the actual OCCT sub-shape, usable with `extrude`, `revolve`, etc.
- **New `face-offset`, `face-fillet`, `face-chamfer`**: Face-level operations (2D offset/fillet/chamfer on face edges)
- **New `face-workplane`**: Create a workplane from a selected face
- **New OCCT bindings**: `BRepAdaptor_Surface` with `GeomAbs_SurfaceType` detection, edge curve geometry access (radius, axis, length)
- **Index stability**: Uses `TopTools_IndexedMapOfShape` for O(1) stable sub-shape indexing
- **New REPL integration tests**: Filter → extract → operate workflows

## Capabilities

### New Capabilities

- `face-query`: Face metadata extraction — type, area, normal, axis, radius, center for each face of a shape
- `edge-query`: Enriched edge metadata — adds length, radius, axis, center to existing edge-info
- `subshape-extraction`: get-face and get-edge functions that extract an OCCT sub-shape by stable index and wrap it as operable ShapeData
- `face-operations`: face-offset, face-fillet, face-chamfer, face-workplane operations on extracted faces

### Modified Capabilities

None — all capabilities are additive.

## Impact

- **New OCCT bindings**: `BRepAdaptor_Surface` + `GeomAbs_SurfaceType`; curve geometry access (Circle::Radius, Line::Direction); curve length utility
- **`src/cad.rs`**: New `FaceInfo` struct, `face_info_json()`, enriched `EdgeInfo`, `get_nth_face()`, `get_nth_edge()`, face operation wrappers
- **`src/main.rs`**: New FFI bridge functions `rust_face_info`, `rust_get_face`, `rust_get_edge`, `rust_face_offset`, `rust_face_fillet`, `rust_face_chamfer`, `rust_face_workplane`
- **`bridge/bridge.c`**: ~7 new JANET_FN functions registered in `cad_register_functions`
- **`boot.janet`**: New wrapper functions `face-info`, `get-face`, `get-edge`, `face-offset`, `face-fillet`, `face-chamfer`, `face-workplane`, convenience `faces`/`edges`
- **Testing**: Unit tests for extraction functions; REPL integration tests for filter→extract→operate chaining
