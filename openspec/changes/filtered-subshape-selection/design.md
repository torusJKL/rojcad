## Context

rojcad wraps OCCT shapes in a `ShapeData` type that lives on Janet's GC heap. Currently a `ShapeData` is always a whole shape (solid, face, wire) created by a CAD operation — there is no way to enumerate or extract sub-shapes (faces, edges) from an existing shape. Edge metadata exists via `edge-info` but is limited to `type`, `start`, `end`. Face metadata does not exist at all.

Operations like `extrude` and `revolve` call `expect_face()` internally, so they already accept a Face-shaped `ShapeData` — but there is no way to obtain a Face sub-shape to pass to them.

The existing bridge pattern (Rust → FFI → C JANET_FN → Janet wrapper) is well-established. This change adds new functions following the same pattern.

## Goals / Non-Goals

**Goals:**

- Provide `face-info` returning per-face metadata structs (index, type, area, normal, axis, radius, center)
- Enrich `edge-info` with `:length`, `:radius`, `:axis`, `:center` fields
- Provide `get-face` / `get-edge` to extract a sub-shape by stable index as an operable `rojcad/shape`
- Provide `face-offset`, `face-fillet`, `face-chamfer`, `face-workplane` operations
- Use `TopTools_IndexedMapOfShape` for O(1) stable indexing
- New sub-shapes are invisible by default (not viewer-registered)
- All new functions follow the existing Janet-idiomatic functional style — no build123d chaining API

**Non-Goals:**

- No changes to existing `extrude`, `revolve`, `fillet`, `chamfer` signatures
- No face → solid fillet/chamfer (that's for solid operations, not 2D face edge operations)
- No `thicken` or `shell` operations
- No parametric naming or persistent topology IDs across shape modifications
- No modifications to the viewer selection system

## Decisions

### D1: Metadata structs via JSON bridge (not Janet abstract types)

`face-info` and `edge-info` return plain Janet structs decoded from JSON, following the existing `edge-info` pattern. This avoids GC pressure from N sub-shape allocations when the user only needs metadata for filtering.

**Alternatives considered:**
- **Return `rojcad/shape` abstracts directly**: Allocates N ShapeData objects per query, wasteful for filtering-only use cases. Rejected.
- **Return C structs via abstract type**: More performant but breaks the existing pattern and makes Janet manipulation harder (no `filter`/`map` on struct keys). Rejected.

### D2: Index-based extraction via TopTools_IndexedMapOfShape

`get-face` and `get-edge` take a parent shape and a 1-based index. Internally they build a `TopTools_IndexedMapOfShape` for O(1) lookup, then wrap the found sub-shape in a new `ShapeData`.

**Alternatives considered:**
- **Linear `nth()` on explorer iterator**: O(n) per call, and explorer order could theoretically differ between calls. `TopTools_IndexedMapOfShape` provides guaranteed stable indices. Selected.
- **Return sub-shapes embedded in metadata structs**: Metadata would carry the actual `rojcad/shape` abstract, but this couples querying and extraction. See D1.

### D3: get-face/get-edge returns new ShapeData, not cached

Each call to `get-face` allocates a fresh Janet GC abstract wrapping a clone of the OCCT sub-shape handle. The sub-shape is not registered in the viewer registry and has `visible = false`.

**Rationale:** Sub-shapes are cheap to allocate (OCCT handles are reference-counted smart pointers). Caching would require a global sub-shape registry with lifetime management complexity. The caller typically creates 1–3 sub-shapes per workflow.

### D4: Separate face-level operation names

Face operations use distinct names: `face-offset`, `face-fillet`, `face-chamfer`, `face-workplane`. This avoids ambiguity with existing solid-level `fillet`/`chamfer` and wire-level `wire-offset`/`wire-fillet`/`wire-chamfer`.

**Rationale:** Although OCCT's `Face::fillet()` and `Solid::fillet_edges()` are different algorithms (2D vs 3D), reusing the name `fillet` for faces would be confusing when the existing `fillet` takes a solid + edge indices.

### D5: Workplane from face via C bridge

`face-workplane` returns a workplane value that can be passed to `sketch`. The Workplane is already wrapped as `rojcad/sketch` abstract in the sketch module — we follow that same pattern.

## Risks / Trade-offs

- **OCCT binding surface**: Wrapping `BRepAdaptor_Surface` and curve geometry access requires adding CXX bindings in `opencascade-sys`, which touches the vendored OCCT dependency. If the bindings are wrong, they fail at link time. → Mitigation: Model new bindings on existing `BRepAdaptor_Curve` wrappings which are proven.

- **Index stability across shape modifications**: If a user calls `face-info`, modifies the shape (fillet, cut, etc.), then calls `get-face` with the old indices, the result is wrong. → Mitigation: This is expected — the shape topology has changed. Document that indices are only valid for the current shape state. The user must re-query after modification.

- **Sub-shape lifetime**: A `get-face`d sub-shape holds a reference to internal OCCT topology (via the handle). If the parent shape is dropped, the OCCT data stays alive until all sub-shape handles are dropped. → Mitigation: This is correct OCCT handle semantics. No action needed.

- **Curve length computation**: OCCT curve length via `GCPnts_AbscissaPoint` or `BRepGProp::CurveLength` may be expensive for BSpline edges. → Mitigation: Length is computed eagerly in `face_info_json`/`edge_info_json` only once. For edges with complex curves, the computation cost is amortized over all operations.
