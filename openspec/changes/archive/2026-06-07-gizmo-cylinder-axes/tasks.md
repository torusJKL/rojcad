## 1. Cylinder Mesh Generation

- [x] 1.1 Add `generate_cylinder(center: Vec3, tip: Vec3, color: [f32; 4], radius: f32, segments: u32) -> Vec<GizmoVertex>` function to `src/viewer/gizmo.rs`
- [x] 1.2 Add `generate_cylinder()` call to `build_static_vertices()` in place of `generate_line_quad()`

## 2. Origin Hub Sphere

- [x] 2.1 Add gray hub sphere at `Vec3::ZERO` in `build_static_vertices()`, after all three cylinders

## 3. Cleanup

- [x] 3.1 Remove `generate_line_quad()` function (keep `LINE_WIDTH` constant for cylinder radius and hub sphere)
- [x] 3.2 Verify: `just check` passes, `just lint` (pre-existing warnings only), `just test` (45/45 pass)
