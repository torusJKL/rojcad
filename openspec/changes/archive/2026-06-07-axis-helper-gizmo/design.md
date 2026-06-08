## Context

The viewer renders CAD shapes via wgpu + winit on a background thread. The current axis indicator (`AxisRenderer`) is three world-space colored line segments at origin (RGB = XYZ), with cone tips rendered via a separate pipeline lacking a view-projection matrix. The `OrbitCamera` stores yaw/pitch/radius and computes the view matrix each frame. Click handling (`handle_click`) uses CPU ray-triangle picking against shape meshes.

## Goals / Non-Goals

**Goals:**
- Axis gizmo in the top-right corner of the viewer
- Three colored lines from center to +X/+Y/+Z, each ending in a colored sphere with a dark camera-facing letter label (X, Y, Z)
- No mouse interaction with the gizmo — view navigation via keyboard shortcuts
- `Ctrl+1`: toggle front (+Z) / back (-Z) — 500ms ease-in-out fly-to
- `Ctrl+7`: toggle top (+Y) / bottom (-Y) — 500ms ease-in-out fly-to
- `Ctrl+3`: toggle left (-X) / right (+X) — 500ms ease-in-out fly-to
- Animation switches main camera to orthographic projection on completion
- Gizmo rotates to match the main camera orientation
- Letters rendered in 3D gizmo space at the sphere surface, using a dedicated depth buffer for proper occlusion (spheres hide letters behind them)
- Remove the old `AxisRenderer`, cone tips, and associated pipeline/buffers

**Non-Goals:**
- No changes to existing shape rendering, edge rendering, grid, or selection
- No changes to the REPL/Janet bridge
- No configuration options for the gizmo from Janet
- No click or hover interaction with the gizmo
- No view-cube or scene-graph panel

## Decisions

### Decision 1: Direct swapchain compositing via viewport/scissor

The gizmo renders directly onto the swapchain in a second render pass after the main scene. A separate depth texture (same size as the swapchain) is cleared each frame and used only for the gizmo pass. `set_viewport` and `set_scissor_rect` restrict rendering to the top-right corner region.

| Approach | Complexity | Notes |
|---|---|---|
| Direct swapchain + viewport/scissor (chosen) | Low | Single framebuffer, depth is separate |
| Offscreen texture + blit | Medium | Extra copy, needed for proper alpha compositing |

### Decision 2: Keyboard-only navigation (no click/hover)

Instead of the planned click-to-navigate, the gizmo is purely visual. All view changes happen via keyboard shortcuts (`Ctrl+1`/`Ctrl+3`/`Ctrl+7`). Modifier state is tracked via `WindowEvent::ModifiersChanged` and stored as `ModifiersState` in `ViewerState`.

### Decision 3: Camera animation via yaw/pitch interpolation

`CameraAnimation` struct added to `ViewerState`:

```rust
struct CameraAnimation {
    active: bool,
    start_yaw: f64,
    start_pitch: f64,
    target_yaw: f64,
    target_pitch: f64,
    elapsed: f64,  // seconds
    // duration: 0.5s
}
```

Ease-in-out: `t * t * (3.0 - 2.0 * t)`. Animation stops on any manual mouse drag. On completion, the camera switches to orthographic projection.

Keyboard view pairs:

| Shortcut | Primary | Opposite | Yaw/Pitch (primary) |
|---|---|---|---|
| Ctrl+1 | +Z (front) | -Z (back) | π/2, 0 |
| Ctrl+7 | +Y (top) | -Y (bottom) | 0, π/2 |
| Ctrl+3 | -X (left) | +X (right) | π, 0 |

Pressing the same shortcut toggles between primary and opposite. Pressing a different shortcut goes to its primary view.

### Decision 4: Letters as 3D billboard geometry (not SDF texture)

Axis labels are rendered as polygon mesh geometry (filled quads forming X/Y/Z shapes) positioned in 3D gizmo space at the sphere center, with a forward bias placing them 0.005 units in front of the sphere surface. They share the same vertex buffer as lines and spheres and are rebuilt every frame to billboard toward the camera.

| Approach | Quality | Complexity |
|---|---|---|
| Polygon mesh billboard (chosen) | Good at fixed size | Low (no texture/sampler needed) |
| SDF font texture | Smooth at any size | Medium (needs atlas + sampler) |

### Decision 5: Depth testing for letter occlusion

A separate `Depth32Float` texture, same size as the swapchain, is created for the gizmo pass. Cleared to 1.0 each frame. The pipeline uses `CompareFunction::Less` with `depth_write_enabled: true`. Letters at the sphere surface are naturally occluded when a different sphere is in front of them.

### Decision 6: GizmoRenderer owns all gizmo state

`GizmoRenderer` struct fields:
- Pipeline, uniform buffer, bind group
- Vertex buffer (rebuilt every frame — includes lines, spheres, and camera-facing letters)
- Depth texture + view
- Viewport size and stored view-projection matrix
- Hovered sphere index (always `None` currently, field retained for future use)

`ViewerState` gains: `gizmo_renderer`, `gizmo_depth`, `gizmo_depth_view`, `gizmo_viewport_size`, `gizmo_margin`, `animation`, `keyboard_view`, `modifiers`.

Gizmo viewport size is scaled by `window.scale_factor()` for high-DPI support.

### Decision 7: No secondary (negative-axis) spheres

Since there is no click interaction, the three spheres opposite the axis labels (at -X, -Y, -Z) serve no purpose and are omitted. Only 3 spheres (at +X, +Y, +Z) with their connecting lines are rendered.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| **Animation conflicts** — User rotates during animation | Stop animation on any manual mouse drag |
| **Letter depth bias** — Letter may z-fight with sphere surface | Use `LETTER_DEPTH_BIAS = CIRCLE_RADIUS + 0.005` (letter is 0.005 units in front of sphere surface) |
| **Depth texture size** — Gizmo depth texture is full-resolution, wastes memory | Acceptable for typical window sizes; on resize the depth texture is recreated |
| **Keyboard shortcut conflicts** — Ctrl+1/3/7 may conflict with system or Janet REPL shortcuts | Currently no conflicts; Ctrl+0 is used for Janet REPL server |
