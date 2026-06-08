## Context

The viewer's `SurfaceDrawer::render()` uses a vertex layout of interleaved `position: vec3<f32>` + `normal: vec3<f32>` (the `MeshVertex` struct, 24 bytes stride). However, `CadMesh::new()` uploads only positions (`mesh.vertices`) to the GPU buffer — a flat `Vec<[f32;3]>` with 12 bytes per element.

This mismatch means the GPU reads every other vertex's position as a normal, and every other vertex falls out-of-bounds → zeros → geometry collapses toward the origin. The sphere makes this dramatically visible because its vertices are densely packed; a box with 8 vertices happens to work by luck since few indices reference the "missing" logical vertices.

## Goals / Non-Goals

**Goals:**
- Fix the vertex buffer to contain interleaved position+normal data matching the GPU's `MeshVertex` layout
- All tessellated shapes render correctly (not just spheres)
- No change to the GPU pipeline, shaders, or vertex layout
- All existing tests continue to pass

**Non-Goals:**
- Changing the MeshData struct or tessellation pipeline (the OCCT data extraction is correct)
- Optimizing vertex upload (can be revisited later)
- Adding normal visualization or debug overlays

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Where to interleave | `CadMesh::new()` | Single point of construction; keeps `extract_mesh()` returning flat arrays for other consumers (STL export, ray picking) |
| Interleave strategy | `iter().zip().map()` | Simple, zero allocations beyond the final buffer; normals.len() == vertices.len() is guaranteed by the mesher |

## Upstream Audit

The `opencascade-rs` crate (v0.1 at rev `7e8d78a`) has a suspicious-looking loop in `crates/opencascade/src/mesh.rs:97-100`:

```rust
// TODO(bschwind) - Why do we start at 1 here?
for i in 1..(normal_array.Length() as usize) {
    let normal = ffi::poly::Poly_Triangulation_Normal(triangulation, i as i32);
    normals.push(dvec3(normal.X(), normal.Y(), normal.Z()));
}
```

**Audit conclusion**: The loop is correct, despite the confusing comment. `normal_array` is allocated with `TColgp_Array1OfDir_new(0, face_point_count)` giving it indices `0..face_point_count` and `Length() = face_point_count + 1`. The loop `1..Length()` therefore iterates `face_point_count` times. `Poly_Triangulation_Normal` uses OCCT's standard 1-based indexing, so starting at `1` is correct — `normal_array` itself is a ghost allocation (unused), only `Poly_Triangulation_Normal` is read.

No upstream patch needed. The debug assertion in `CadMesh::new()` guards against any future mismatch.

## Risks / Trade-offs

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Normals count differs from vertices count | Low | Debug assertion in `CadMesh::new()` validates `normals.len() == vertices.len()` before zipping |
| Regression on edge/line rendering | None | Edge rendering uses a separate pipeline with its own vertex buffer — unaffected |
