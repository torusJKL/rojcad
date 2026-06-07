## ADDED Requirements

### Requirement: Vertex buffer contains interleaved position and normal data

The GPU vertex buffer SHALL contain interleaved `position: vec3<f32>` + `normal: vec3<f32>` data (24 bytes per vertex) matching the `MeshVertex` layout used by the render pipeline. Positions and normals from the same mesh index MUST occupy adjacent memory in the buffer.

#### Scenario: Sphere mesh renders without geometric corruption

- **WHEN** a sphere is created via `(make-sphere 5.0)` and rendered in the viewer
- **THEN** the sphere surface appears smooth and continuous with no gaps, no triangles collapsing to the origin, and no ~180° void

#### Scenario: All tessellated shapes use correct vertex layout

- **WHEN** any shape (box, cylinder, boolean result) is tessellated and uploaded to the GPU
- **THEN** every vertex in the buffer contains both position and normal data, and the GPU pipeline reads exactly `sizeof(MeshVertex)` bytes per vertex
