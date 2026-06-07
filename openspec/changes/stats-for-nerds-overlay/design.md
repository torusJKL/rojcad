## Context

rojcad's wgpu-based viewer renders 3D CAD shapes in a single window with a grid, gizmo (top-right), and mouse-driven orbit/pick. The viewer communicates with the REPL thread via a shared `ShapeRegistry` (Arc<RwLock>) and atomic globals for toggles. Camera state (yaw, pitch, radius) is held in `OrbitCamera` in `ViewerState` but never surfaced to the user.

Design constraints from the existing architecture:
- Custom text rendering via a glyph atlas is complex and fragile (vertical alignment issues, font loading)
- egui provides production-quality text rendering, layout, and floating windows out of the box
- egui-wgpu integrates directly with wgpu render passes
- egui-winit translates winit events to egui input
- Toggles use `AtomicBool` statics in `src/types.rs`
- Janet bridge functions follow a 3-layer pattern: `bridge.c` (JANET_FN) → extern in `src/bridge.rs` → Rust impl in `src/main.rs`

## Goals / Non-Goals

**Goals:**
- Render a text overlay in a draggable egui floating window
- Show camera state: yaw (degrees), pitch (degrees), zoom/radius, projection mode
- Show a human-readable view name by matching (yaw, pitch) against presets
- Show total/visible/hidden shape counts from the shared registry
- Show selected shape ID
- Show total triangle and vertex counts from visible meshes
- Show FPS and frame time using a smoothed ring buffer
- Show toggle states: back edges (X), projection (P), overlay itself
- Toggle overlay with `Ctrl + Shift + Alt + S` and Janet function `stats-overlay`
- Feed winit events to egui before camera/pick handling

**Non-Goals:**
- Complex egui panels (settings, shape browser) — stats window only
- Custom theming — use egui default look
- Remembering window position across sessions

## Decisions

### Decision 1: Use egui over custom glyph atlas

| Criterion | egui | Custom glyph atlas |
|-----------|------|-------------------|
| Text rendering | Built-in (font loading, atlas, layout, AA) | 700+ lines custom code |
| Floating/draggable window | Built-in | Not feasible |
| Vertical alignment | Automatic | Requires manual `font_ascent` alignment |
| Input handling | Built-in (hover, click) | Manual hit-test needed |
| Future UI growth | Trivial to add widgets | Each widget is a new pipeline |
| Dependencies | +3 (egui, egui-wgpu, egui-winit) | +1 (ab_glyph) |

**Chosen**: egui. The vector of future UI features (settings panels, view cube, toolbars) makes egui's upfront cost worthwhile. The current custom approach requires ~700 lines for what egui provides with zero layout code.

### Decision 2: Feed events to egui before camera/pick

egui needs first access to mouse/keyboard events so it can handle hover states, window dragging, and text input. The winit event handler first calls `egui_state.on_window_event()`, and if egui consumed the event (e.g., hovering over the stats window), we skip camera orbit/pick processing for that event.

### Decision 3: Separate render pass with egui-wgpu

egui renders into a dedicated render pass after the gizmo, using `LoadOp::Load` and no depth/stencil. egui-wgpu manages its own vertex buffers and textures internally.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **egui version compat** — egui-wgpu must match wgpu version | Build breakage | Pin exact versions in Cargo.toml |
| **Input conflicts** — egui consuming events interferes with camera controls | Camera drag when clicking outside window | Check `egui_ctx.wants_pointer_input()` before camera handling |
| **Performance** — egui allocates each frame | Frame time jitter | egui reuses allocations internally; negligible for a simple stats window |
