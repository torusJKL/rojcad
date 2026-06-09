## Context

rojcad's data model already has `color: Option<[f64;3]>` on both `ShapeData` (Janet GC-managed) and `ShapeEntry` (viewer registry), but the field is only populated with `None` — never set, never read.

The viewer renders with hardcoded grey in `shader.wgsl`, with a separate blue pipeline for selection. Edge colors are globally tunable via atomics; surface colors have no equivalent.

OCCT's `Compound::from_shapes()` is used internally in `src/text.rs` but not exposed.

## Goals / Non-Goals

**Goals:**
- Expose `Compound::from_shapes()` as a Janet `compound` function for grouping 1+ shapes
- Add per-shape color with `color` (setter, mutates in place, returns same shape) and `get-color` (getter)
- Support `:color` keyword on `compound` for one-shot color-on-group
- Wire the `color` field through to GPU so the viewer renders per-mesh color instead of hardcoded grey
- Default color when `None`: grey (0.75, 0.75, 0.75) — matches current behavior

**Non-Goals:**
- Color propagation through transforms (e.g., `(-> (sphere 5) (color [1 0 0]) (translate 0 0 10))` loses color — known limitation, not addressed here)
- `:color` keyword on primitive constructors (`box`, `sphere`, etc.)
- Application-level groups (scene graph) — `compound` creates an OCCT `Compound`, which is a single topological entity
- Per-subshape colors within a compound (compound = one shape = one color)

## Decisions

### 1. `compound` is a thin C bridge function, not a Janet-only wrapper
`Compound::from_shapes()` lives in opencascade-rs. The simplest path: C bridge takes a tuple of shapes + eager + hide, passes them to Rust. Color is set post-hoc by the Janet wrapper, not passed through the C bridge.

```
Janet `compound` wrapper
  → collects shapes, parses :color/:eager/:hide
  → calls C `_cad_compound(shapes_tuple, eager, hide)`
    → Rust `make_compound(&shapes)`
  → if :color, calls C `_cad_set_color(shape, r, g, b)`
    → Rust `set_color(shape, r, g, b)`
  → returns shape
```

**Note**: Cannot use `wrap-c-fn` for `compound`. Janet's standard library defines `comp` as function composition (in `upstream.janet`). A C registration as `"comp"` would be overwritten. Register as `"compound"` in C bridge, capture manually.

### 1a. Upstream Janet `comp` is preserved
Because we name our function `compound` (not `comp`), Janet's built-in function composition `(comp f g h)` remains available to users in the REPL. No shadowing.

### 2. `color` mutates metadata only — no geometry
`set_color()` updates the `ShapeData.color` field and, if registered, updates `ShapeEntry` in the global `ShapeRegistry` and bumps the generation counter. The OCCT `Shape` inside `ShapeData` is untouched.

### 3. Viewer: per-mesh color via a second bind group
Current pipeline uses bind group 0 for view-proj uniforms. We add bind group 1 with `var<uniform> mesh_color: vec4<f32>`. Each `CadMesh` owns a 16-byte uniform buffer + bind group, created at mesh-build time from `entry.color` (or default grey).

```
SurfaceDrawer pipeline layout:
  group(0) binding(0): var<uniform> uniforms: Uniforms        (view-proj)
  group(1) binding(0): var<uniform> mesh_color: vec4<f32>     (per-mesh)

render():
  for each CadMesh:
    pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    pass.set_pipeline(normal or highlight);
    pass.set_bind_group(1, &mesh.color_bind_group, &[]);      // NEW
    pass.set_vertex_buffer(...);
    pass.set_index_buffer(...);
    pass.draw_indexed(...);
```

Selection highlight (`fs_highlight`) ignores mesh_color and stays hardcoded blue — selection is a transient UI state, not a shape attribute.

### 4. Single-shape `(compound a)` returns `a` unchanged
In Rust `make_compound`, if `shapes.len() == 1`, return the shape as-is without wrapping. This avoids unnecessary compound overhead and matches user expectation.

### 5. `(compound)` with 0 shapes returns an error
No meaningful default for an empty compound. Error at the Rust level.

### 6. Color values are clamped to [0, 1]
Uses the existing `pack_color`/`unpack_color` semantics (16-bit per channel). Out-of-range values are clamped.

## Risks / Trade-offs

- **[Risk] Color is lost through transforms**: `translate`, `rotate`, etc. create new `ShapeData` from scratch (color = None). If users chain `color` before a transform, the color disappears. Fixing this requires copying metadata in each transform function — proportional effort, deferred.
- **[Risk] Many small GPU buffers**: N meshes = N uniform buffers (16 bytes each). For typical CAD scenes (< 500 shapes) this is ~8 KB — negligible. Mitigation: could switch to a single storage buffer indexed by draw order, but not worth the complexity now.
- **[Trade-off] Bind group per mesh**: Creating N bind groups is slightly heavier than a single storage buffer. Bind groups are lightweight (descriptors, not allocations) and are rebuilt infrequently (only on registry changes).
- **[Risk] Viewer must handle None color**: Currently `entry.color` is always None. After this change, it may be Some or None. The viewer builder maps None → `[0.75, 0.75, 0.75]` (grey). This is backward-compatible.
