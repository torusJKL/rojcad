## Context

Edge rendering in the wgpu viewer uses a monolithic approach: all shapes' edge polylines are flattened into a single vertex buffer, then rendered via two pipelines (solid front, dashed back) with a hardcoded dark grey color (0.05, 0.05, 0.05). There is no per-shape or per-selection distinction for edges.

The `SurfaceDrawer` already handles selection awareness — it checks `selected_id` and picks between `render_pipeline` (grey) and `highlight_pipeline` (blue) per mesh. The `EdgeDrawer` has no equivalent logic.

Selection state is tracked via `ViewerState.selected_id: Option<ShapeId>` and the global `LAST_SELECTION` atomic.

Control from the Janet REPL currently uses global `AtomicU64`/`AtomicBool` statics (e.g., `LAST_SELECTION`, `REGISTRY_GENERATION`) exposed through C bridge functions.

## Goals / Non-Goals

**Goals:**
- Edge color changes from dark grey to light grey for unselected shapes
- Edge color changes to light blue for the selected shape
- Back edges of the selected shape are also light blue (dashed)
- Two independent toggles: `edge-toggle-inactive` hides/shows non-selected shape edges, `edge-toggle-active` hides/shows selected shape edges
- Query functions to read current toggle state
- All toggles are runtime-only (no persistence)
- Existing `X` key back-edge toggle continues to work

**Non-Goals:**
- Per-shape edge colors (only selected vs unselected distinction)
- Per-shape edge style (thickness, dash pattern)
- Edge thickness control from REPL
- Edge color customization from REPL
- Persistent toggle state across restarts

## Decisions

### 1. Split edge buffer into active and inactive

| Approach | Complexity | GPU overhead | Code clarity |
|----------|-----------|-------------|-------------|
| Single buffer + per-vertex color attribute | Medium (needs new vertex format + shader change) | Minimal | Moderate — per-vertex branching in shader |
| Split into two buffers (chosen) | Low | One extra buffer allocation per frame | High — clear separation of concerns |

**Chosen**: Split the edge vertex buffer into two — `inactive_edge_buffer` and `active_edge_buffer`. The viewer rebuilds both each frame from `visible_shapes()`, partitioning by whether the shape ID matches `selected_id`. This mirrors how the code already iterates all visible shapes to build edges.

### 2. Four pipelines instead of two

`EdgeDrawer` gains two new pipeline variants:

```
Current:                      Proposed:
┌──────────────────────┐      ┌──────────────────────┐
│ solid_pipeline       │      │ inactive_solid       │ grey front
│ dashed_pipeline      │      │ inactive_dashed      │ grey back
└──────────────────────┘      │ active_solid         │ blue front
                              │ active_dashed        │ blue back
                              └──────────────────────┘
```

The fragment shaders in `edge.wgsl` accept a `@group(1) @binding(0) color: vec4<f32>` uniform instead of hardcoded color values, so both inactive and active variants can share the same shader module with different uniform bind groups. Alternatively, add a uniform buffer — but for just a single vec4 color, a simple approach is two separate entry points in the shader with hardcoded colors, matching the existing pattern in `shader.wgsl` (which has both `fs_main` and `fs_highlight`).

**Chosen**: Two additional fragment shader entry points (`fs_inactive`, `fs_active`) in `edge.wgsl`, matching the existing pattern in `shader.wgsl`. This avoids adding uniform infrastructure for a single constant color. The pipelines use `build_pipeline()` with different fragment entry points.

### 3. Toggle state as global atomics

Following the established pattern of `LAST_SELECTION` and `REGISTRY_GENERATION`:

```rust
// src/types.rs
pub static SHOW_INACTIVE_EDGES: AtomicBool = AtomicBool::new(true);
pub static SHOW_ACTIVE_EDGES: AtomicBool = AtomicBool::new(true);
```

These are read by the viewer thread each frame in `render()` and set by the C bridge functions when called from Janet.

No channel messages needed — the channel is for shape data flow, not viewer configuration.

### 4. Janet API functions

Four functions exposed through `bridge/bridge.c`:

| Janet function | C impl | Behavior |
|---|---|---|
| `(edge-toggle-inactive)` | `cad_edge_toggle_inactive` | Toggle `SHOW_INACTIVE_EDGES`, return new value as boolean |
| `(edge-toggle-active)` | `cad_edge_toggle_active` | Toggle `SHOW_ACTIVE_EDGES`, return new value as boolean |
| `(edge-inactive-show?)` | `cad_edge_inactive_showing` | Return `SHOW_INACTIVE_EDGES` as boolean |
| `(edge-active-show?)` | `cad_edge_active_showing` | Return `SHOW_ACTIVE_EDGES` as boolean |

The `?` suffix on query functions follows Janet convention (already used for `visible?`).

### 5. Render order

Meshes must render BEFORE edges, so edges overlay on top of mesh surfaces (overcoming the silhouette-only visibility of 1-pixel lines).

```
Each frame (render()):
1. Grid + axes (unchanged)
2. Cone tips (unchanged)
3. Mesh surfaces (depth test Less, writes depth, includes selection highlight)
4. Inactive edges (if SHOW_INACTIVE_EDGES):
   a. dashed back edges (depth Greater, negative bias)
   b. solid front edges (depth Less, negative bias)
5. Active edges (if SHOW_ACTIVE_EDGES and selected_id is Some):
   a. dashed back edges (depth Greater, negative bias)
   b. solid front edges (depth Less, negative bias)
```

Edge rendering uses a negative depth bias (`constant: -4, slope_scale: -2.0`) to pull edge fragments slightly toward the camera, ensuring they pass the depth test against the mesh surface they overlay. Edges write no depth (`depth_write_enabled: false`) so they don't occlude each other.

### 6. Instanced line rendering (screen-space quads)

Instead of wgpu's native `LineList` (which is limited to 1-pixel-wide lines), each edge segment is rendered as a 4-vertex `TriangleStrip` quad, expanded in the vertex shader to a screen-space rectangle of configurable thickness (`uniforms.thickness` in NDC units, default 0.004 ≈ 3 pixels at 1024 width). This technique gives crisp, controllable-width lines.

### 8. Synthetic wireframe for curved shapes

OCCT topological edges only capture the BRep seam/intersection curves. For curved primitives like spheres, this produces a single seam edge (a 180° arc), which is insufficient for visual wireframe.

| Approach | Result on sphere | Result on box |
|---|---|---|
| OCCT topological edges only | 1 seam arc (180°, invisible from most angles) | 12 clean edges ✓ |
| Mesh triangle edges (all) | Dense triangulation wireframe ✗ (thousands of edges) | Ugly face diagonals ✗ |
| Synthetic bounding-sphere circles (chosen) | Clean equator + meridian ✓ | Not applied (box has 12 edges > threshold of 8) ✓ |

**Decision**: In `tessellate_and_update` (cad.rs), after extracting topological edges, count them. If fewer than `SYNTHETIC_WIREFRAME_THRESHOLD` (8), generate two synthetic circle polylines from the mesh bounding box:

- **Equator**: A horizontal circle in the XZ plane at the bounding box's Y-center, with radius derived from the X and Z extents.
- **Meridian**: A vertical circle in the XY plane at the bounding box's Z-center, with radius derived from the X and Y extents.

Both circles use 32 segments for smooth appearance. The synthetic wireframe is generated per-frame when the registry is rebuilt, so it automatically follows shape modifications.

This ensures:
- Spheres get a clean two-circle wireframe ✓
- Boxes (12 topological edges) keep only their sharp outline ✓
- Boolean results with few edges also get the synthetic fallback ✓
- No dense triangle edges or face diagonals ✗

### 9. Runtime thickness and color control

Thickness and per-state colors are stored as atomics (`AtomicU64`) and read by the viewer each frame when building the uniform buffer.

| Parameter | Atomic | Access |
|---|---|---|
| Thickness | `EDGE_THICKNESS: AtomicU64` (f64 bits) | `(edge-thickness 0.008)` / `(edge-thickness)` |
| Inactive color | `INACTIVE_EDGE_COLOR: AtomicU64` (packed u64) | `(edge-color-inactive 0.7 0.7 0.7)` |
| Active color | `ACTIVE_EDGE_COLOR: AtomicU64` (packed u64) | `(edge-color-active 0.4 0.6 1.0)` |

Colors are packed as `u64`: 16 bits per channel (R << 32 | G << 16 | B). Pack/unpack functions in `src/types.rs`. The viewer's `EdgeDrawer::update_uniforms()` reads these atomics and writes the full `EdgeUniforms` struct (view_proj, inactive_color, active_color, thickness) to the GPU uniform buffer.

Shader fragment entry points reference `uniforms.inactive_color` and `uniforms.active_color` instead of hardcoded values.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|---|
| **Depth bias too large or too small** — wrong bias causes edges to either be hidden by the mesh (too small) or punch through other geometry (too large). | `constant: -4, slope_scale: -2.0` is a conservative starting point. Unlike `LineList`, the instanced quad approach is clearly visible at silhouette edges regardless of bias — the bias only affects the widest part of the quad where it overlies the mesh face. |
| **Color packing precision** — 16-bit per channel (65535 levels) is generous for display purposes; visible banding is unlikely. | Standard 8-bit displays cannot distinguish adjacent 16-bit levels. |
| **Janet function naming inconsistency** — `visible?` uses `?` suffix for predicate, but toggle functions return a value. | `edge-inactive-show?` follows the `?` convention for queries. Toggle functions return the new state as a boolean (Janet `true`/`false`), which is a standard Janet pattern. |
