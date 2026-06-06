## 1. Core Architecture

- [x] 1.1 Create `GizmoRenderer` struct in `src/viewer/gizmo.rs` with pipeline, buffers, uniforms, depth texture
- [x] 1.2 Create `src/viewer/gizmo.wgsl` with vertex/fragment shaders
- [x] 1.3 Add `pub mod gizmo` to `src/viewer/mod.rs`
- [x] 1.4 Wire gizmo renderer into `ViewerState` and initialise in `resumed()`

## 2. Camera Animation System

- [x] 2.1 Define `CameraAnimation` struct with active flag, start/target yaw/pitch, elapsed time
- [x] 2.2 Implement ease-in-out interpolation (cubic: `t * t * (3.0 - 2.0 * t)`)
- [x] 2.3 Add `CameraAnimation` to `ViewerState`, update each frame in `render()` with dt
- [x] 2.4 Stop animation on any manual mouse input (drag)
- [x] 2.5 Switch main camera to orthographic projection when animation completes

## 3. Gizmo 3D Geometry

- [x] 3.1 Build axis line quads (3 lines: center → +X, +Y, +Z) with RGB colors
- [x] 3.2 Build UV spheres (14×8 segments) at +X, +Y, +Z with matching colors
- [x] 3.3 Build letter meshes (X, Y, Z as polygon bars) positioned at sphere surface with forward bias for camera-facing billboard
- [x] 3.4 Add per-frame vertex buffer rebuild (lines + spheres + letters combined) to support billboard and hover updates

## 4. Rendering Pipeline

- [x] 4.1 Gizmo renders in a second render pass after the main scene (swapchain color attachment, LoadOp::Load)
- [x] 4.2 Dedicated depth texture (Depth32Float, swapchain-sized) cleared to 1.0 per frame
- [x] 4.3 Use `set_viewport` / `set_scissor_rect` to restrict gizmo to top-right corner
- [x] 4.4 Pipeline uses `CompareFunction::Less` + `depth_write_enabled: true` for proper sphere-letter occlusion

## 5. Keyboard Shortcuts

- [x] 5.1 Track `ModifiersState` via `WindowEvent::ModifiersChanged`
- [x] 5.2 `Ctrl+1`: toggle front (+Z) / back (-Z) view with 500ms animation
- [x] 5.3 `Ctrl+7`: toggle top (+Y) / bottom (-Y) view with 500ms animation
- [x] 5.4 `Ctrl+3`: toggle left (-X) / right (+X) view with 500ms animation
- [x] 5.5 Track `keyboard_view` in ViewerState for toggle state (pressing same shortcut flips; different shortcut resets)

## 6. Removal of Old Axis System

- [x] 6.1 Remove `AxisRenderer` struct and all usage
- [x] 6.2 Remove `build_axis_cones`, `build_cone_pipeline`, cone vertex functions
- [x] 6.3 Remove cone/axis fields from `ViewerState`
- [x] 6.4 Remove old axis and cone rendering calls from render pass

## 7. Cleanup and Polish

- [x] 7.1 DPI-scaling: viewport size multiplied by `window.scale_factor()`
- [x] 7.2 No mouse hover or click interaction with gizmo (hover always `None`)
- [x] 7.3 Colors updated to final values (#F03752, #76B316, #2D89F0)
- [x] 7.4 Gizmo removes hover state (set_hovered still available but unused)

## 8. Verification

- [x] 8.1 Build passes with no errors or relevant warnings
- [x] 8.2 Run and verify gizmo appears at top-right corner (manual)
- [x] 8.3 Verify gizmo rotates with camera orbit (manual)
- [x] 8.4 Test Ctrl+1/3/7 shortcuts trigger correct animated views (manual)
- [x] 8.5 Test toggle behavior (same shortcut flips, different shortcut resets) (manual)
- [x] 8.6 Test manual rotation cancels animation (manual)
- [x] 8.7 Verify old axis indicator and cones are gone (manual)
