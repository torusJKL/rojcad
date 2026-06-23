## Context

The viewer renders shapes as triangle meshes (surface) + edge polylines (instanced screen-space quads). Click detection currently only intersects the ray against mesh triangles — there is no edge-level picking. Edge data (polylines, topological count, per-edge metadata) already exists in `ShapeEntry` and is available from OCCT. The `edge-info` Janet API returns edge type/start/end, and `highlight-edge` visually highlights specific edges by index. Fillet/chamfer already accept edge indices via `:e` keyword.

The bridge has four layers: OCCT → Rust CAD → Rust FFI (extern "C") → C (bridge.c JANET_FN) → Janet (boot.janet wrappers). Selection flows: viewer click → atomics → C bridge poll → Janet `poll-selection` → `poll-viewer` fiber prints to REPL.

`boot.janet:1086` already maintains a `shape-bindings` table mapping symbol → shape abstract, updated by `my-eval` on every `def`/`set`. This provides reverse name lookup.

## Goals / Non-Goals

**Goals:**
- Click-to-select individual edges in the 3D viewer
- Multi-edge selection (Shift=add, Ctrl=toggle, plain=replace)
- Edge selection also selects parent shape
- Edge events flow through the existing polling mechanism
- REPL output shows edge index and shape name
- New `(edge-selection)` query and `(fillet-selected)`/`(chamfer-selected)` helpers

**Non-Goals:**
- GPU-based picking (CPU screen-space is sufficient and simpler)
- Vertex/face picking
- Edge hover effects (highlight on mouse-over without click)
- Spork REPL eval tracking (only `my-eval` paths track shape bindings)

## Decisions

### Decision 1: Screen-space edge polyline projection vs. GPU picking

**Chosen: Screen-space projection (CPU)**
Alternatives considered: GPU pick buffer (render edge IDs to offscreen framebuffer, read back on click).

Rationale: Edge polylines are already in `ShapeEntry.edge_polylines` — the same data used for rendering. Projecting polyline segments to screen coordinates and measuring distance from click is straightforward, follows the existing CPU pick pattern (`pick.rs`), and requires no GPU pipeline changes. Performance is adequate: even a 1000-edge model at 20 segments/edge = 20k segment checks; each is a few dot products and one perspective divide.

**Threshold**: A constant `EDGE_CLICK_THRESHOLD_PX = 6.0` pixels is used (tuned pragmatically: 3px was too tight on high-DPI screens, 12px was generous but imprecise, 6px balances both). The original formula based on `EDGE_THICKNESS` was abandoned because rendered edge width (sub-pixel) is unrelated to usable click target size.

### Decision 2: Edge always wins over shape on ambiguous clicks

Edges are rendered with depth bias (constant: -4, slope_scale: -2.0) and appear on top of mesh surfaces. When a click is within threshold of any edge polyline, the closest edge (by z-depth after projection) wins. The mesh pick is only used as fallback when no edge is hit within threshold.

### Decision 3: Separate atomics for edge selection events

New atomics `LAST_EDGE_SHAPE_ID` (u64), `LAST_EDGE_INDEX` (i32, -1 = none), and `LAST_EDGE_ACTION` (u8, 0 = none, 4 = selected, 5 = deselected) sit alongside existing `LAST_SELECTION`/`LAST_SELECTION_ACTION`. When `_cad_poll_selection_raw` is called, edge atomics are checked first (higher priority). The C bridge returns 3-element tuples `[action, shape_id, edge_index]` for all events:

| Action | Meaning | shape_id | edge_index |
|--------|---------|----------|------------|
| 1 | shape selected | sid | -1 |
| 2 | shape deselected | sid | -1 |
| 3 | all cleared | MAX | -1 |
| 4 | edge selected | sid | edge_idx |
| 5 | edge deselected | sid | edge_idx |

Edge atomics are polled first; if consumed, shape atomics remain for the next poll cycle.

### Decision 4: Three-category edge rendering with dedicated selected pipeline

Edges are split into three categories during the generation-triggered GPU rebuild:

| Category | Color | Source | Pipeline |
|---|---|---|---|
| Inactive | Grey (uniform) | All edges not highlighted | `inactive_solid_pipeline` (`Less`) |
| API-highlighted | Blue (uniform `active_color`) | `highlighted_edges` or selected shape | `active_solid_pipeline` (`Less`) |
| User-selected | Orange (hardcoded) | `selected_edges` (click result) | `selected_solid_pipeline` (`Always`) |

`state.selected_edges: HashMap<ShapeId, HashSet<usize>>` is tracked separately from `state.highlighted_edges`. The edge building loop uses an `if-else if-else` chain to mutually exclude edges from the three buffers.

The selected pipeline uses `CompareFunction::Always` so it always passes the depth test and renders on top of both inactive and active edges. This is necessary because all three categories share the same depth buffer and the active edges use `Less` (so a second draw at equal depth fails).

`REGISTRY_GENERATION` is bumped on every selection change to trigger GPU buffer rebuild.

### Decision 5: Reverse name lookup via `shape-bindings` table

The existing `shape-bindings` table in boot.janet maps `symbol → shape`. A `shape-name` helper does a reverse value scan to find the symbol. Fallback: when no binding exists (shape created without `def`), display `<shape {id}>` using the numeric shape_id.

### Decision 6: Synthetic edges excluded from picking

`ShapeEntry` gains a `topo_edge_count: usize` field tracking the number of real topological edges (before synthetic wireframe is appended). The edge picker skips indices ≥ `topo_edge_count`. Synthetic edges are visual-only and can't be filleted/chamfered.

### Decision 7: Multi-edge selection mirrors shape selection modifiers

| Modifier | Click on edge | Click on empty space |
|----------|---------------|---------------------|
| Plain | Replace edge sel, select shape | Clear all (edges + shapes) |
| Shift | Add this edge, add shape | No-op |
| Ctrl | Toggle this edge, add shape | No-op |

### Decision 8: Hardcoded orange in dedicated fragment shaders

User-selected edges use dedicated shader entry points (`fs_selected_solid` / `fs_selected_dashed`) that return a hardcoded `vec4<f32>(1.0, 0.5, 0.0, 1.0)` (orange) rather than reading from the uniform buffer.

Alternatives considered:
- **Modifying the uniform's `active_color` mid-frame**: `queue.write_buffer` writes are asynchronous — the bind group captures state at `set_bind_group` time, and the write may not be visible before the draw executes. Abandoned.
- **Adding `selected_color` to `EdgeUniforms`**: Requires WGSL struct layout change + uniform buffer resize + new shader entry points + pipeline registration. Equivalent complexity to the chosen approach but loses the benefit of the color being independent of the uniform.

The hardcoded approach means the selected-edge color is not runtime-tunable (unlike inactive/active colors which come from `EdgeUniforms`). This is acceptable since the distinction between "user-selected" and "API-highlighted" is semantic, not aesthetic.

### Decision 9: Click threshold is a constant, not derived from edge thickness

The rendered edge width (`EDGE_THICKNESS`) is typically sub-pixel (~0.5px on 1080p) and unusable as a click target. The threshold was determined empirically by testing at 3px (too tight), 12px (too generous), and settling on 6px. This is independent of edge styling.

### Decision 10: Hidden edge rejection via CPU ray-mesh occlusion test

When `SHOW_BACK_EDGES` is false (default), occluded edges are not selectable. An edge is considered hidden if its midpoint is behind the mesh surface from the camera's viewpoint.

The occlusion test (`is_edge_hidden` in `pick.rs`) casts a ray from the camera origin through the edge's midpoint and checks the same shape's mesh for closer intersections. If a mesh triangle is closer than the midpoint (by more than 0.1 world units), the edge is occluded. This mirrors what the GPU depth test (Greater) does during rendering — an edge fragment passes the Greater test only when behind the mesh.

Performance: O(edge_triangles) per picked edge. Acceptable for interactive use since this only runs once per click, for the single closest edge candidate. The mesh data is already resident in CPU memory.

The rendered edge width (`EDGE_THICKNESS`) is typically sub-pixel (~0.5px on 1080p) and unusable as a click target. The threshold was determined empirically by testing at 3px (too tight), 12px (too generous), and settling on 6px. This is independent of edge styling.

## Risks / Trade-offs

- **Performance on dense models**: 1000+ edges each with 50+ polyline segments = 50k segment checks per click. Mitigation: early-out by testing shape bounding boxes first; cache projected polylines per frame.
- **Ambiguous edge hits at corners**: a click at a vertex could be within threshold of multiple edges. Mitigation: pick closest by z-depth after segment projection (frontmost wins).
- **`shape-bindings` only tracks `my-eval` paths**: spork netrepl eval does not update the table. Mitigation: acceptable for now; spork is a secondary REPL. A future improvement could hook into spork's eval path as well.
- **Reverse name lookup is O(n)**: iterates all shape bindings. Mitigation: negligible at interactive scale (< 100 shapes). Could add a reverse index later if needed.
- **`Always` depth test on selected edges**: ensures orange overlays blue but means selected edges are always drawn regardless of occlusion. For convex shapes seen through other geometry, the orange edge might be visible when it should be hidden. Mitigation: acceptable trade-off — edge selection is an interactive operation and visibility of the selection indicator takes priority over occlusion correctness.
- **Janet bootstrap mode compile-time visibility limit**: `janet_cfuns` entries past ~94 and `def`/`put core-env` bindings created after a certain point in `boot.janet` are not visible to subsequent `compile` calls in the TCP REPL. This is a fundamental limitation of bootstrap mode — `janet_def` adds to the environment table at runtime but doesn't register the symbol in the compiler's internal lookup table. Mitigation: `(edge-selection)` is defined early in `boot.janet` (line 19) where `defn` creates compile-time-visible bindings; it accesses the C function at runtime via `(get core-env '_edge-selection-raw)`. The TCP REPL cannot compile `(edge-selection)` directly but `--eval` and GUI REPL work correctly.
