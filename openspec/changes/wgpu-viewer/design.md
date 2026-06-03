## Context

rojcad is a headless parametric CAD system with an embedded Janet DSL and OCCT backend via `opencascade-rs`. Currently shapes are created, modified, and exported entirely through a TCP REPL — no visual feedback. The project compiles OCCT from source with `TKService`, `TKV3d`, and `TKOpenGl` linked but inert.

OCCT provides AIS (Application Interactive Services) for 3D visualization, but its Qt5 integration doesn't work on Wayland, and Qt6 is a heavy dependency. The alternative is to tessellate OCCT BRep shapes into triangle meshes and render them via a modern GPU API.

The `opencascade-rs` crate already exposes `Shape::mesh()` which returns in-memory triangle data (`Mesh { vertices, uvs, normals, indices }`) via OCCT's `BRepMesh_IncrementalMesh`. The upstream repo has a working reference viewer using wgpu + winit that renders these meshes and edge polylines.

We will add the viewer as an optional companion: auto-spawned by default, suppressible via `--headless`. The viewer runs on a background winit thread communicating with the REPL thread via channels and a shared shape registry.

## Goals / Non-Goals

**Goals:**
- Interactive 3D viewport with orbit/pan/zoom camera
- Mesh surface rendering of OCCT shapes (tessellated via `shape.mesh()`)
- Edge wireframe rendering (topological edges via `shape.edges()`, sampled as polylines)
- Dashed back-edge rendering (toggle with `X` key) for visual clarity
- Planar helper grid on XZ plane (Y=0, standard engineering convention)
- World-space axis indicator (RGB = XYZ, with cone tips)
- Shape-level selection: click selects the nearest shape intersection via CPU ray-triangle test
- Visual highlighting of selected shape (tinted mesh + bright edges)
- Automatic viewer update: shapes created, modified, or hidden in the REPL update instantly
- Selection events propagated back to the Janet REPL as callbacks
- `--headless` CLI flag to suppress viewer startup (backward compatible)

**Non-Goals:**
- Face/edge/vertex level selection (shape-level only)
- Measurement tools, section cuts, or dimension annotations
- View cube or scene graph panel
- PBR materials or shadows (flat/smooth shaded is sufficient)
- Export from viewer (already available via REPL)
- WASM hot-reload or WASM component model (the upstream viewer's experimental features)

## Decisions

### Decision 1: wgpu + winit over OCCT AIS + Qt6

| Criterion | wgpu + winit | AIS + Qt6 |
|-----------|-------------|-----------|
| Wayland support | ✅ Native (winit) | ❌ Requires Qt6+Wayland workaround |
| Build complexity | Light (cargo deps) | Heavy (Qt6 build system, moc) |
| Binary size | +3-5 MB | +30-60 MB |
| Integration | Pure Rust, no FFI for rendering | Requires OCCT Graphic3d C++ pipeline |
| Selection | Need to build (CPU ray-triangle) | Built-in (AIS picking) |
| Maturity for CAD | Basic (mesh + edges) | Full (exact display, HLR, etc.) |

**Chosen**: wgpu + winit. The selection feature is the main trade-off, but CPU ray-triangle intersection against CAD-scale meshes is fast enough and straightforward to implement.

### Decision 2: Background thread for viewer (Linux)

winit on Linux (Wayland/X11) can run on a non-main thread. This allows the existing REPL to remain on the main thread unchanged. The viewer thread is spawned at startup and communicates via `mpsc` channels.

On macOS, winit requires the main thread. We add a compile-time check: if `cfg!(target_os = "macos")`, we flip the architecture (winit on main, REPL on background). For now, Linux is the primary target.

**Thread model:**

```
REPL thread (main)                     Viewer thread (background)
┌──────────────────────┐              ┌─────────────────────────┐
│  Janet eval loop     │              │  winit event loop       │
│  TCP server          │   channel    │   │                     │
│  cad.rs operations   │◄════════════►│   ▼                     │
│  ShapeRegistry       │  mesh data   │  wgpu renderer          │
│  (Arc<RwLock>)       │  selections  │   • mesh surfaces       │
└──────────────────────┘              │   • edge wires          │
                                      │   • grid + axes         │
                                      │   • highlight pass      │
                                      │   • CPU ray picking     │
                                      └─────────────────────────┘
```

**Channel protocol:**
```
REPL → Viewer:
  - UpdateShapes(Vec<(ShapeId, MeshData, EdgeData)>)
  - RemoveShape(ShapeId)
  - ClearAll

Viewer → REPL:
  - ShapeSelected(ShapeId)
  - ShapeDeselected
  - ViewerClosed
```

### Decision 3: CPU ray-triangle selection

| Approach | Complexity | Performance | Precision |
|----------|-----------|-------------|-----------|
| CPU ray-triangle (Möller–Trumbore) | Low (~100 lines) | O(n) per click, fine for <1M triangles | Exact f64 (mesh data from OCCT) |
| GPU color-buffer picking | Medium | O(1) per click | Limited by render target resolution |
| OCCT BRepExtrema | High (needs OCCT C++ wrapper) | N/A | Exact geometric |

**Chosen**: CPU ray-triangle. The mesh data is already on the CPU (from `shape.mesh()`). For CAD models with thousands of triangles, iterating all triangles per click is not a bottleneck. We implement the Möller–Trumbore algorithm in Rust using `glam` (already a dependency).

### Decision 4: Edge rendering via instanced polylines (same approach as viewer crate)

The upstream viewer crate renders edges by:
1. Iterating `shape.edges()` to get topological edges
2. Sampling each edge's underlying curve via `BRepAdaptor_Curve` + `GCPnts_UniformAbscissa`
3. Building instanced line segments with round caps/joins using a dedicated WGSL shader

This produces smooth, accurate edge curves (not mesh triangulation edges). We replicate the same approach. The line rendering uses the technique described in [Instanced Line Rendering](https://wwwtyro.net/2019/11/18/instanced-lines.html).

Two-pass technique for back edges:
1. Pass 1: Depth function `Greater`, dashed line style → back edges
2. Pass 2: Depth function `Less`, solid line style → front edges

### Decision 5: Grid and axes as line instances

The grid is a static set of instanced line segments (major/minor/central lines on the XZ plane) fed into the same edge pipeline with different color uniforms. The axes are three additional line segments (stems) plus a small cone mesh per axis tip. This avoids dedicated GPU pipelines.

### Decision 6: Shape registry as shared state

A new `ShapeRegistry` type (backed by `Arc<RwLock<HashMap<ShapeId, ShapeEntry>>>`) lives in `src/types.rs`. After every CAD operation, the shape is inserted/updated in the registry. The viewer thread reads the registry each frame to rebuild mesh/edge GPU data on change.

`ShapeEntry` contains:
```rust
struct ShapeEntry {
    shape_id: u64,
    mesh: Option<Mesh>,          // cached tessellation
    edge_polylines: Vec<Vec<DVec3>>, // cached edge polylines
    visible: bool,
    color: Option<[f64; 3]>,
}
```

Tessellation is computed lazily: when the shape is first viewed, or on demand after modification.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Thread safety of OCCT** — OCCT is not thread-safe for concurrent shape modification. | Data corruption or crashes. | All OCCT operations remain on the REPL thread. The render thread only reads pre-tessellated mesh data (plain Rust structs, no OCCT handles). |
| **winit on non-main thread** — Platform-specific behavior. | Crash on macOS. | `cfg!(target_os = "macos")` guard; document as Linux-only for now. Mac support is a future concern. |
| **Large models** — Meshing a complex assembly could block the REPL. | REPL feels unresponsive. | Defer mesh computation to a background task pool (future optimization). For initial implementation, mesh synchronously — acceptable for typical CAD model sizes. |
| **wgpu version churn** — wgpu 24 is the current stable, but API changes rapidly. | Build breakage on upgrade. | Pin exact version in `Cargo.toml`. Track upstream viewer for migration patterns. |
| **Selection misses on thin features** — If a triangle is smaller than a pixel, ray might miss. | Shapes with very thin features may be unselectable. | Fall back to bounding sphere intersection if triangle test misses. |
