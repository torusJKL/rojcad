## Why

The wgpu viewer uses `CullMode::Back` with `FrontFace::Ccw` for mesh surface rendering. For a closed solid like a box, this means faces whose outward normal points away from the camera are culled — their inside surface is treated as a back face. This creates visual holes in the geometry: looking at a box from a diagonal angle, the bottom and left-side faces disappear, revealing the interior and far-side faces beyond.

CAD viewers typically show all geometry regardless of orientation. Disabling backface culling matches user expectations: every surface is visible, even if the inside faces appear darker (ambient-only lighting) due to inward-pointing normals.

## What Changes

- Change `cull_mode` from `Some(wgpu::Face::Back)` to `None` on the surface render pipeline

## Capabilities

### New Capabilities

*None — this changes an existing rendering setting.*

### Modified Capabilities

- `3d-viewer`: Mesh surfaces now render both sides. Inside faces show ambient-only lighting instead of being invisible.

## Impact

- **Modified files**: `src/viewer/app.rs` — one line change
- **No new dependencies**
- **No breaking changes**
- **No performance impact** (disable culling is free on modern GPUs)
