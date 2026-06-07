## Why

The viewer currently lacks a navigational axis helper. Users must orbit randomly to find views, with no visual reference for orientation or a way to jump to standard orthographic views. This hurts usability for CAD work where precise view alignment is essential.

## What Changes

- Replace the current world-space axis indicator (three colored lines with cone tips at origin) with a top-right corner axis gizmo
- The gizmo shows three colored axis lines (RGB = XYZ) from center, each ending in a colored sphere with the axis letter (X, Y, Z) as a camera-facing 3D label
- No interactive click/hover on the gizmo — view changes are keyboard-driven
- Keyboard shortcuts: `Ctrl+1` (front/back toggle), `Ctrl+7` (top/bottom toggle), `Ctrl+3` (left/right toggle) trigger 500ms animated camera fly-to
- Add camera animation system (ease-in-out interpolation of yaw/pitch)
- Orthographic gizmo camera copies main camera rotation; dedicated depth buffer for proper sphere-to-letter occlusion
- Remove old AxisRenderer and cone pipeline

## Capabilities

### New Capabilities
- `view-axis-gizmo`: 3D axis orientation widget with keyboard-navigated orthographic view shortcuts

### Modified Capabilities
- *(none — this is a new capability)*

## Impact

- **File changes**: `src/viewer/app.rs`, `src/viewer/camera.rs`, `src/viewer/gizmo.rs` (new), `src/viewer/gizmo.wgsl` (new), `src/viewer/mod.rs`
- **Dependencies**: No new external deps (wgpu + glam already available)
- **Removals**: `AxisRenderer` struct, `build_axis_cones`, `cone_pipeline`, `axis_cone_*` fields in `ViewerState`
- **Non-breaking**: Existing camera controls, shape rendering, and selection unaffected
