## Why

rojcad is currently headless — shapes exist only as data in the REPL. Without visual feedback, parametric modeling is slow and error-prone. Adding an interactive 3D viewer gives real-time visual feedback for every CAD operation, making the system usable for actual design work. OCCT's built-in AIS viewer is the natural choice but requires Qt6 for Wayland support. A wgpu + winit viewer provides a lighter, Wayland-native alternative that works with the existing Rust toolchain.

## What Changes

- Add an interactive 3D viewer window using wgpu + winit that opens automatically on startup
- Support `--headless` CLI argument to suppress the viewer (restoring headless mode)
- Tessellate OCCT shapes via `shape.mesh()` and render triangle meshes in the viewer
- Render topological edges as instanced polylines (solid front edges, dashed back edges)
- Render a planar helper grid on the XZ plane and world-space axis indicator (RGB = XYZ)
- Implement shape-level selection via CPU ray-triangle intersection with highlight rendering
- Sync the viewer with the REPL — shapes created, modified, or hidden update automatically
- Wire selection events back to the Janet REPL

## Capabilities

### New Capabilities
- `3d-viewer`: Interactive wgpu + winit 3D viewport showing OCCT shapes with mesh surfaces, edge wireframes, helper grid, and axis indicator.
- `viewer-selection`: Shape-level click selection with visual highlighting, driven by CPU ray-triangle intersection.
- `viewer-repl-sync`: Bidirectional synchronization between the REPL and viewer — shapes update automatically, selection events flow back to Janet.

### Modified Capabilities

*None — this is additive; no existing capability changes its requirements.*

## Impact

- **New dependencies**: `wgpu` (v24), `winit` (v0.30)
- **New module**: `src/viewer/` (~1500 lines across 7 files)
- **Modified files**: `Cargo.toml` (add deps), `src/main.rs` (CLI arg, viewer spawn), `src/cad.rs` (shape registration), `src/types.rs` (shape registry)
- **No breaking changes**: `--headless` preserves full backward compatibility
- **Build time impact**: Minor — wgpu compiles shaders and spir-v tooling on first build
