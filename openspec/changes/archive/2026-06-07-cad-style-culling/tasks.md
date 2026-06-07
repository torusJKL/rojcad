## 1. Disable backface culling on surface pipeline

- [x] 1.1 Change `cull_mode: Some(wgpu::Face::Back)` to `cull_mode: None` in `SurfaceDrawer::build_pipeline` at `src/viewer/app.rs:229`
- [x] 1.2 Run `cargo build` to verify compilation
- [x] 1.3 Run `cargo test` to verify no regressions
- [x] 1.4 Launch viewer, create a box (`(box 10 10 10)`), rotate camera to verify all faces are visible from any angle
