## 1. Edge Fragment Shader Colors

- [x] 1.1 Add `fs_inactive` entry point to `edge.wgsl` that returns light grey `(0.7, 0.7, 0.7)`
- [x] 1.2 Add `fs_active` entry point to `edge.wgsl` that returns light blue `(0.4, 0.6, 1.0)`
- [x] 1.3 Update existing `fs_solid` and `fs_dashed` to be inactive variants (grey) or remove them

## 2. EdgeDrawer Pipeline Expansion

- [x] 2.1 Add `active_solid_pipeline` and `active_dashed_pipeline` fields to `EdgeDrawer`
- [x] 2.2 In `EdgeDrawer::new()`, build the two new pipelines using `fs_active` fragment entry with `CompareFunction::Less` (solid) and `CompareFunction::Greater` (dashed)
- [x] 2.3 Rename existing `solid_pipeline` → `inactive_solid_pipeline`, `dashed_pipeline` → `inactive_dashed_pipeline`
- [x] 2.4 Update `EdgeDrawer::render()` to accept separate active and inactive buffers and render them independently (respecting toggle flags)

## 3. Edge Buffer Splitting in Viewer Render Loop

- [x] 3.1 In `ViewerState::render()`, after reading visible shapes from registry, partition edge polylines into two groups: active (matching `selected_id`) and inactive (everything else)
- [x] 3.2 Build separate `active_edge_buffer` and `inactive_edge_buffer` GPU vertex buffers
- [x] 3.3 Add `active_edge_vertex_buffer` and `active_edge_num_vertices` fields to `ViewerState`
- [x] 3.4 Update the dirty-tracking section to rebuild both buffers on generation change
- [x] 3.5 Wire the two buffers and toggle state into the render pass

## 4. Toggle State Atomics

- [x] 4.1 Add `pub static SHOW_INACTIVE_EDGES: AtomicBool` and `pub static SHOW_ACTIVE_EDGES: AtomicBool` to `src/types.rs`, both initialized to `true`
- [x] 4.2 Add Rust `extern "C"` functions in `src/main.rs` to toggle and query each atomic: `rust_edge_toggle_inactive`, `rust_edge_toggle_active`, `rust_edge_inactive_showing`, `rust_edge_active_showing`

## 5. Janet C Bridge Functions

- [x] 5.1 Add extern forward declarations for the four new Rust functions in `bridge/bridge.c`
- [x] 5.2 Implement `cad_edge_toggle_inactive` JANET_FN — toggle `SHOW_INACTIVE_EDGES`, return new state as boolean
- [x] 5.3 Implement `cad_edge_toggle_active` JANET_FN — toggle `SHOW_ACTIVE_EDGES`, return new state as boolean
- [x] 5.4 Implement `cad_edge_inactive_showing` JANET_FN — return `SHOW_INACTIVE_EDGES` as boolean
- [x] 5.5 Implement `cad_edge_active_showing` JANET_FN — return `SHOW_ACTIVE_EDGES` as boolean
- [x] 5.6 Register all four functions in `cad_register_functions` JanetReg array

## 6. Instanced Line Rendering (Screen-Space Quads)

- [x] 6.1 Write `vs_line` vertex shader that expands each segment to a screen-space quad (TriangleStrip)
- [x] 6.2 Add `grid_pipeline` (LineList, vs_main) separate from edge pipelines (TriangleStrip, vs_line)
- [x] 6.3 Build `SegmentInstance` arrays instead of plain vertex buffers for edge segments
- [x] 6.4 Change render order: mesh surfaces first, then edges with negative depth bias

## 7. Edge Style Runtime Control

- [x] 7.1 Extend `Uniforms` in `edge.wgsl` with `inactive_color`, `active_color`, `thickness`
- [x] 7.2 Add `EdgeUniforms` Rust struct and update `EdgeDrawer::update_uniforms()` to read from atomics
- [x] 7.3 Add globals `EDGE_THICKNESS`, `INACTIVE_EDGE_COLOR`, `ACTIVE_EDGE_COLOR` to `src/types.rs`
- [x] 7.4 Add Rust extern functions: `rust_edge_set_thickness`, `rust_edge_get_thickness`, `rust_edge_set_color_inactive`, `rust_edge_set_color_active`
- [x] 7.5 Add JANET_FN implementations: `cad_edge_thickness`, `cad_edge_color_inactive`, `cad_edge_color_active`
- [x] 7.6 Add docstrings to all new Janet functions

## 8. Build and Test

- [x] 8.1 Run `just build` to verify compilation
- [x] 8.2 Run `just test-unit` to verify no regressions
- [ ] 8.3 Launch viewer, create shapes, verify edges are light grey and visible from all angles
- [ ] 8.4 Click a shape to select it, verify edges turn light blue
- [ ] 8.5 Test `(edge-toggle-inactive)` — edges disappear/reappear on non-selected shapes
- [ ] 8.6 Test `(edge-toggle-active)` — edges disappear/reappear on selected shape
- [ ] 8.7 Test `(edge-inactive-show?)` and `(edge-active-show?)` return correct states
- [ ] 8.8 Test `(edge-thickness 0.008)` — verify edges get thicker
- [ ] 8.9 Test `(edge-color-inactive 0.9 0.9 0.9)` — verify inactive edge color changes
- [ ] 8.10 Test `(edge-color-active 1.0 0.0 0.0)` — verify selected edge color changes to red
- [ ] 8.11 Verify `X` key back-edge toggle still works with both active and inactive edges
