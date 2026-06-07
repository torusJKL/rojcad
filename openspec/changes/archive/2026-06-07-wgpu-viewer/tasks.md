## 1. Dependencies and Project Setup

- [x] 1.1 Add `wgpu = "24"` and `winit = "0.30"` to `Cargo.toml`
- [x] 1.2 Create `src/viewer/` module directory with `mod.rs` and verify it compiles
- [x] 1.3 Add `--headless` CLI argument to `src/main.rs` via `std::env::args` parsing

## 2. Shape Registry (Shared State)

- [x] 2.1 Add `ShapeId` type alias (`u64`) and `ShapeEntry` struct to `src/types.rs` (shape_id, mesh, edge_polylines, visible, color)
- [x] 2.2 Implement `ShapeRegistry` as `Arc<RwLock<HashMap<ShapeId, ShapeEntry>>>` in `src/types.rs`
- [x] 2.3 Add thread-safe methods: `register()`, `update()`, `remove()`, `visible_shapes()`, `shape_by_id()`
- [x] 2.4 Modify `cad.rs` operations to register/update shapes in the registry after creation
- [x] 2.5 Wire shape visibility changes (`hide`/`show`) to update the registry

## 3. Viewer Launch Infrastructure

- [x] 3.1 Define channel types in `src/viewer/mod.rs` (ViewerToRepl, ReplToViewer enums)
- [x] 3.2 Implement `ViewerHandle` struct (spawn + join + channel senders)
- [x] 3.3 Implement `spawn_viewer()` function that creates winit event loop on background thread
- [x] 3.4 Wire viewer spawn in `main.rs`: if `--headless` flag absent, call `spawn_viewer()`
- [x] 3.5 Implement graceful shutdown: viewer close sends `ViewerClosed` event, main thread cleans up

## 4. Camera Module

- [x] 4.1 Implement `OrbitCamera` struct in `src/viewer/camera.rs` with target, radius, orientation, projection
- [x] 4.2 Implement `pan()`, `zoom()`, `rotate()` methods
- [x] 4.3 Implement `matrix() -> Mat4` computing combined view-projection matrix
- [x] 4.4 Implement perspective/orthographic toggle

## 5. wgpu + winit Window and Event Loop

- [x] 5.1 Implement wgpu instance, adapter, device, and surface creation in `src/viewer/app.rs`
- [x] 5.2 Implement swapchain creation and configuration
- [x] 5.3 Wire winit event loop on background thread with `Event::WindowEvent` handling
- [x] 5.4 Map mouse events (left-drag orbit, middle-drag pan, right-drag zoom, scroll zoom)
- [x] 5.5 Map keyboard events (P/O toggle projection, X toggle back-edges, Escape close)
- [x] 5.6 Implement resize handler for window size changes

## 6. Surface Rendering Pipeline

- [x] 6.1 Write mesh vertex shader in `src/viewer/shader.wgsl` (transform position, pass normal/uv)
- [x] 6.2 Write mesh fragment shader (lambertian diffuse shading with neutral gray material)
- [x] 6.3 Implement `CadMesh` struct: build vertex/index GPU buffers from `opencascade::Mesh` data
- [x] 6.4 Implement `SurfaceDrawer` struct wgpu render pipeline (vertex layout, bind groups, render pass)
- [x] 6.5 Implement `SurfaceDrawer::render()` that draws all shape meshes in one pass

## 7. Edge Rendering Pipeline

- [x] 7.1 Implement edge polyline extraction: iterate `shape.edges()`, sample curves as polylines
- [x] 7.2 Implement `SegmentInstance` struct (endpoints A/B, cumulative length for dashing)
- [x] 7.3 Write edge vertex shader (instanced line rendering with round caps/joins)
- [x] 7.4 Write edge fragment shader (solid/dashed line style via `dist` discard)
- [x] 7.5 Implement `EdgeDrawer` struct with solid and dashed render pipelines
- [x] 7.6 Implement two-pass edge rendering: pass 1 (dashed, depth Greater), pass 2 (solid, depth Less)
- [x] 7.7 Integrate `X` key to toggle back-edge dashed pass on/off

## 8. Grid and Axis Indicator

- [x] 8.1 Implement grid line generation: major/minor/central lines on XZ plane
- [x] 8.2 Build grid as static `Vec<SegmentInstance>` for the instanced line pipeline
- [x] 8.3 Build axis stems as three line instances (RGB = XYZ)
- [x] 8.4 Build axis cone tip meshes (8 triangles each, centered at axis endpoints)
- [x] 8.5 Render grid + axes each frame alongside shape edges

## 9. Selection and Highlighting

- [x] 9.1 Implement Möller–Trumbore ray-triangle intersection function in `src/viewer/pick.rs`
- [x] 9.2 Implement `pick_shape()`: cast ray from click position against all visible shapes
- [x] 9.3 Wire left-click to trigger picking and set/clear selected shape ID
- [x] 9.4 Implement highlight rendering: tint selected shape mesh blue + bright edges
- [x] 9.5 Send `ShapeSelected`/`ShapeDeselected` events through Viewer→Repl channel

## 10. Viewer-REPL Sync Integration

- [x] 10.1 In the winit event loop tick: read `ShapeRegistry`, detect changes, rebuild GPU data
- [x] 10.2 Implement dirty tracking: only rebuild meshes/edges for shapes that changed
- [x] 10.3 Wire `ShapeRegistry.visible_shapes()` to determine what to render each frame
- [x] 10.4 Register a viewer selection callback in Janet: `(on-select (fn [s] (print "selected: " s)))`

## 11. Testing and Verification

- [x] 11.1 Run `cargo build` and fix any compilation errors
- [x] 11.2 Run `cargo build --release` and verify viewer window opens on startup
- [x] 11.3 Run `cargo run --release -- --headless` and verify headless mode works with no window
- [x] 11.4 Connect `nc 127.0.0.1 9365`, create shapes, verify they appear in the viewer
- [x] 11.5 Test orbit/pan/zoom camera controls in the viewer
- [x] 11.6 Test shape selection by clicking on shapes in the viewport
- [x] 11.7 Test back-edge toggle (`X` key) and projection toggle (`P`/`O`)
- [x] 11.8 Test `(hide s)` / `(show s)` and verify viewer updates instantly
- [x] 11.9 Run existing unit tests: `cargo test` — verify nothing is broken
