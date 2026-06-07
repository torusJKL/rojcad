## Context

The viewer's `SurfaceDrawer::build_pipeline` (src/viewer/app.rs:225-233) configures culling as:

```rust
primitive: wgpu::PrimitiveState {
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    ..
}
```

OCCT triangulates each face with CCW winding when viewed from the face's outward-normal direction (standard convention). When the camera views a face from the opposite side — e.g., the bottom of a box when the camera is above — the face's triangle winding appears CW in screen space, making it a back face that gets culled.

## Decision: Disable backface culling

| Approach | Change | Visual result |
|----------|--------|---------------|
| `CullMode::None` (chosen) | One line: `cull_mode: None` | All faces visible; inside faces dark (ambient-only, 0.225 on 0.15 bg) |
| Double-sided lighting | Shader uses `@builtin(front_facing)` to flip normals | All faces correctly lit, but more complex shader change |

For a CAD viewer, showing all geometry with correct ambient occlusion is the standard behavior. The inside faces being darker is actually desirable — it gives depth perception without needing to add proper ambient occlusion.

## What about the highlight pipeline?

Both `render_pipeline` and `highlight_pipeline` call `Self::build_pipeline()` with the same `cull_mode`. A single change applies to both.

## What about the cone pipeline?

The cone pipeline (axis indicators) already has `cull_mode: None` (app.rs:606). No change needed there.

## Risk

None. Disabling culling is strictly additive — more pixels render, never fewer.
