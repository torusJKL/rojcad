## Context

rojcad exposes opencascade-rs CAD operations through a 4-layer architecture:
```
cad.rs (Rust) → main.rs (FFI bridge) → bridge.c (JANET_FN wrappers) → boot.janet (Janet wrappers)
```

Existing `wire-fillet`/`wire-chamfer` operate on 2D wires only, using `Wire::fillet()`/`Wire::chamfer()`. The opencascade-rs `Shape` type provides fillet/chamfer for 3D solids:
- `Shape::fillet(radius)` — fillet all edges
- `Shape::fillet_edges(radius, edges)` — fillet specific edges
- `Shape::chamfer(distance)` / `Shape::chamfer_edges(distance, edges)` — same for chamfer
- `Shape::edges()` — iterates topological edges for selection

The vendored Janet (1.41.2) lacks a JSON parser — no `json.c` in core, no `json/parse` in upstream boot. rojcad has a custom `json-encode` in `boot.janet` for GUI REPL communication. The spork library provides `src/json.c` with native `json/encode` and `json/decode`.

The viewer has a highlight system for shapes (`ReplToViewer::HighlightShape`) using a mpsc channel and `highlighted_shape: Option<ShapeId>` state, but no per-edge highlight.

## Goals / Non-Goals

**Goals:**
- Expose `fillet` and `chamfer` as Janet functions operating on 3D solids/shells
- Support optional edge selection via `:e` keyword (tuple of indices)
- Provide `edge-info` to let users discover edge indices
- Provide `highlight-edge` / `highlight-edge-clear` for visual preview
- Add spork JSON module for `json/decode` and `json/encode`

**Non-Goals:**
- Interactive edge selection in the viewer (click-to-pick) — deferred to follow-up
- Per-edge variable radius/distance (e.g., different fillet radii on different edges) — deferred
- Modifying existing `wire-fillet`/`wire-chamfer` behavior

## Decisions

### 1. Unified fillet/chamfer API (single function, not separate all-edges + selected-edges)

`fillet` and `chamfer` accept an optional `:e` keyword with a tuple of edge indices. Absence means "all edges".

**Alternatives considered:**
- Separate `fillet`/`fillet-edges` — rejected, user wants one function
- `shape-fillet`/`shape-chamfer` — rejected, bare `fillet`/`chamfer` matches conventions of `extrude`, `revolve`, etc.

### 2. Edge selection by integer index

Edges are identified by their 0-based index in the `Shape::edges()` iteration order. Users discover indices via `edge-info`.

**Alternatives considered:**
- Opaque edge handles — adds GC complexity for no benefit
- Interactive viewer selection — deferred to follow-up

### 3. Spork JSON over custom Janet parser or serde_json + C bridge

The spork `json.c` provides native `json/encode` and `json/decode` as a self-contained C module. Adding it to `vendor/core/` requires:
- Removing the `JANET_MODULE_ENTRY` macro (not needed for static linking) and naming the init function `janet_lib_spork_json`
- `build.rs` auto-discovers `.c` files in `vendor/core/` — no build changes needed
- Registering via `bridge::janet_lib_spork_json(env)` in `main.rs`

**Alternatives considered:**
- Writing a Janet-level `json-decode` (~40 lines) — more code to maintain, not reusable
- `serde_json` + C bridge building tables — adds dep + C code, JSON not involved at all
- `serde_json` + Janet decoder — adds dep + Janet code, more moving parts

### 4. `serde_json` for Rust-side EdgeInfo serialization

EdgeInfo is `#[derive(Serialize)]` and serialized to JSON in Rust. The JSON string is passed to `json/decode` on the Janet side.

**Alternatives considered:**
- `#[repr(C)]` struct + C bridge building tables — more C code, edge-specific, no reusability
- `#[repr(C)]` struct — fixed-size string buffers for edge type are awkward in FFI

### 5. Edge highlight via viewer channel extension

Extends `ReplToViewer` enum with `HighlightEdges { shape_id, indices }` and `ClearEdgeHighlight`. Viewer stores `highlighted_edges: HashMap<ShapeId, HashSet<usize>>` and renders highlighted edges in orange (vs blue for shape-level highlight).

## Risks / Trade-offs

- **[Edge index fragility]** Edge indices depend on OCCT's topological iteration order, which could change if the shape is reconstructed. Indices are stable within a single session as long as the shape topology doesn't change. → Mitigation: document that indices should be used immediately after querying, not stored across shape mutations.
- **[spork JSON ABI compatibility]** The spork json.c uses standard Janet C API — should link cleanly against the same Janet version (1.41.2). → Low risk.
- **[JSON performance]** JSON serialize/deserialize round-trip for edge-info is unnecessary overhead for a query function. → Acceptable: edge-info is a debugging/discovery tool, not called in hot paths.
- **[Degenerate geometry crash]** OCCT's `BRepFilletAPI::Shape()` can throw `StdFail_NotDone` or segfault when chamfer/fillet distance exceeds feasibility. OCCT's C++ exceptions cannot be caught by Rust `catch_unwind` because CXX's `extern "C"` bridge wrappers are `noexcept`. → Mitigation: three-layer defense (see design section below).

## Degenerate Geometry Protection

Three layers of defense against OCCT crash from degenerate chamfer/fillet parameters:

```
Layer 1: Pre-validation (cad.rs)
    validate_edge_dimension() checks distance/radius against
    edge lengths before any OCCT call.
    Rejects when value × 2 >= edge_length.
    Uses safe OCCT queries only (start_point/end_point).

        ┌─────────────────────────────────┐
        │  chamfer(fillet) called         │
        │         │                       │
        │         ▼                       │
        │  validate_edge_dimension()       │
        │  (PASS)         (FAIL)→error    │
        │    │                            │
        │    ▼                            │
        │  OCCT Shape::chamfer_edges()    │
        │         │                       │
        │         ▼                       │
        │  validate_has_geometry()        │
        │  (PASS)         (FAIL)→error    │
        │    │                            │
        │    ▼                            │
        │  ShapeData returned to Janet    │
        │         │                       │
        │         ▼                       │
        │  Viewer tessellate_if_needed()  │
        │  sets mesh=None if empty        │
        │         │                       │
        │         ▼                       │
        │  Render: skip if mesh is None   │
        └─────────────────────────────────┘

Layer 2: Post-check (cad.rs)
    validate_has_geometry() tessellates result and checks
    for mesh vertices OR edge polylines.
    Wire shapes pass (edges without mesh are valid).

Layer 3: Viewer guard (types.rs + app.rs)
    tessellate_if_needed() sets mesh = None when
    OCCT returns no vertices/indices.
    Render loop skips entries with mesh.is_none()
    or empty vertex/index buffers.
```
