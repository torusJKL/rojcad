## 1. Fix Vertex Buffer Construction

- [x] 1.1 Add a debug assertion in `CadMesh::new()` that `mesh.normals.len() == mesh.vertices.len()`
- [x] 1.2 Interleave positions and normals into `Vec<MeshVertex>` before uploading to the GPU buffer
- [x] 1.3 Replace `bytemuck::cast_slice(&mesh.vertices)` with `bytemuck::cast_slice(&interleaved)` in the `create_buffer_init` call

## 2. Upstream Normals Loop Audit

- [x] 2.1 Investigate the `TODO(bschwind) - Why do we start at 1 here?` loop in `opencascade-rs` crate's `mesh.rs:97-100`
- [x] 2.2 Document audit conclusion: the loop is correct — OCCT triangulation uses 1-based indexing, the ghost `normal_array` allocation makes `Length()` = `face_point_count + 1`, so `1..Length()` iterates exactly `face_point_count` times

## 3. Verification

- [x] 3.1 Build the project and verify it compiles without warnings
- [x] 3.2 Run `cargo test` to confirm all existing tests pass
- [x] 3.3 Launch the viewer, create a sphere via `(make-sphere 5.0)`, and confirm the mesh renders as a smooth continuous sphere with no gaps or collapsed geometry
- [x] 3.4 Verify that other shapes (box, cylinder, boolean cut/common) also render correctly
