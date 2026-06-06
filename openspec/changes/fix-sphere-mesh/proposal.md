## Why

When creating a sphere via `(make-sphere 5.0)`, the rendered mesh is corrupt — triangles connect to wrong positions and a ~180° gap appears across the surface. The root cause is in the vertex buffer construction, not the OCCT tessellation itself. This makes the viewer unreliable for any shape rendering.

## What Changes

- Fix `CadMesh::new()` in `src/viewer/app.rs` to upload interleaved position+normal vertex data instead of positions-only
- The vertex buffer layout (`MeshVertex` struct) is unchanged — only the data fed into it is wrong
- This is a one-line fix in the hot path (vertex buffer creation), no new dependencies

## Capabilities

### New Capabilities

*None — this is a bug fix, not a new capability.*

### Modified Capabilities

- `3d-viewer`: Mesh rendering was uploading positions-only vertex buffers while the GPU expected interleaved position+normal data. The vertex buffer construction in `CadMesh::new()` must interleave positions and normals to match the `MeshVertex` layout.

## Impact

- **Affected code**: `src/viewer/app.rs` — `CadMesh::new()` method
- **No API changes**: The `MeshData` struct, vertex layout, and public interfaces are unchanged
- **All shapes benefit**: Boxes, booleans, cylinders — any tessellated shape — are affected by this bug, though the sphere makes it most visible
