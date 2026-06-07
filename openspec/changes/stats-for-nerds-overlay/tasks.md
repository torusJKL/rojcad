## 1. Setup — Swap Dependencies

- [x] 1.1 Remove `ab_glyph = "0.2"` from `[dependencies]` in `Cargo.toml`
- [x] 1.2 Add `egui = "0.32"`, `egui-wgpu = "0.32"`, `egui-winit = "0.32"` to `[dependencies]` in `Cargo.toml`
- [x] 1.3 Upgrade `wgpu` from "24" to "25" for egui-wgpu compatibility

## 2. Rewrite stats.rs to use egui

- [x] 2.1 Remove custom glyph atlas, WGSL shader, and all wgpu pipeline code
- [x] 2.2 Implement `Stats` struct with `ui()` method that builds the stats window using egui widgets
- [x] 2.3 Compute and pass stats: yaw/pitch/zoom, view preset name, projection mode, shape counts, triangle/vertex counts, FPS/frame time, toggle states
- [x] 2.4 Use `egui::Window::new("Stats").anchor(Align2::LEFT_TOP, [14.0, 14.0])` for a draggable floating window with `egui::Grid` layout

## 3. Wire egui Into Viewer

- [x] 3.1 Add `egui_ctx: egui::Context`, `egui_state: egui_winit::State`, `egui_renderer: egui_wgpu::Renderer` to `ViewerState`
- [x] 3.2 Initialize in `resumed()`: create context, state (with event_loop), renderer (with device + format)
- [x] 3.3 Feed winit events to egui before camera/pick handling in `window_event()`
- [x] 3.4 In `render()`: build egui UI via `egui_ctx.run(raw_input)`, update textures+buffers, then render with `egui_wgpu::Renderer` in a separate pass after the gizmo pass
- [x] 3.5 Remove all old `StatsOverlay` field types and methods from app.rs
- [x] 3.6 Restore `CloseRequested` and `Resized` event handlers that were accidentally dropped

## 4. Janet Bridge (unchanged)

- [x] 4.1 Add `rust_stats_overlay_toggle()`, `rust_stats_overlay_showing()`, `rust_stats_overlay_set()` functions in `src/main.rs`
- [x] 4.2 Add `JANET_FN(cad_stats_overlay, ...)` in `bridge/bridge.c`
- [x] 4.3 Register `stats-overlay` in `cad_fn_categories` and `cfuns`

## 5. Build & Verify

- [x] 5.1 Run `cargo check` to verify compilation
- [x] 5.2 Run `just fmt` to format the code
- [x] 5.3 Run `just lint` to check clippy
- [x] 5.4 Run `cargo test` to verify existing tests still pass
