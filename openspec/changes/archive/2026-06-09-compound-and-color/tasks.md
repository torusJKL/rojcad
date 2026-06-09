## 1. Rust Core — `make_compound` and `set_color`

- [x] 1.1 Add `make_compound(shapes: &[ShapeData]) -> Result<ShapeData, String>` to `src/cad.rs`
  - 2+ shapes via `Compound::from_shapes()`, 0/1 handled by Janet wrapper
- [x] 1.2 Add `set_color(data: &mut ShapeData, r: f64, g: f64, b: f64)` to `src/cad.rs`
  - Clamp values to [0, 1], update `data.color`, if registered update `ShapeEntry` + bump generation
- [x] 1.3 Add `get_color(data: &ShapeData) -> Option<[f64; 3]>` to `src/cad.rs`
  - Return `data.color` as-is

## 2. C Bridge — Janet-callable functions

- [x] 2.1 Add `_cad_compound` JANET_FN in `bridge/bridge.c`
  - Takes `(shapes_tuple eager hide)`, unpacks shapes from tuple, calls Rust `make_compound`
  - Registered as `"compound"` (NOT `"comp"` — avoids shadowing Janet's built-in `comp` function composition)
- [x] 2.2 Add `_cad_set_color` JANET_FN in `bridge/bridge.c`
  - Takes `(shape r g b)`, calls Rust `set_color`, returns same shape
- [x] 2.3 Add `_cad_get_color` JANET_FN in `bridge/bridge.c`
  - Takes `(shape)`, calls Rust `get_color`, returns 3-element tuple or nil
- [x] 2.4 Register all three in `cad_register_functions()` with docstrings

## 3. Rust FFI — Bridge declarations

- [x] 3.1 Add `extern "C"` declarations for `rust_init_compound`, `rust_set_color`, `rust_get_color` in `src/bridge.rs`
- [x] 3.2 Add FFI bridge functions in `src/main.rs` for each: extract args from C, call `cad::*`, return result

## 4. Janet Wrappers — User-facing API

- [x] 4.1 Add `compound` wrapper in `boot.janet`: variadic, parses :color/:eager/:hide, calls C `_cad_compound`, applies color if :color given
  - Cannot use `wrap-c-fn` because `comp` is a built-in Janet function. Name our function `compound` instead, register as `"compound"` in C bridge, capture manually.
- [x] 4.2 Add `color` wrapper in `boot.janet`: calls C `_cad_set_color`, returns same shape
- [x] 4.3 Add `get-color` wrapper in `boot.janet`: calls C `_cad_get_color`
- [x] 4.4 Register under `cad-operations` meta group

## 5. Viewer — Per-mesh color rendering

- [x] 5.1 Add bind group layout entry at group(1) binding(0) for `var<uniform> mesh_color: vec4<f32>` in `SurfaceDrawer`
- [x] 5.2 Add `color_buffer: wgpu::Buffer` and `color_bind_group: wgpu::BindGroup` to `CadMesh`
- [x] 5.3 Update `CadMesh::new()` (or builder) to accept color, create buffer + bind group
- [x] 5.4 Update mesh builder to read `entry.color` (default grey when None) and pass to CadMesh
- [x] 5.5 Update `SurfaceDrawer::render()` to bind group(1) before each mesh draw
- [x] 5.6 Update `shader.wgsl` `fs_main` to read `mesh_color.rgb` instead of hardcoded `vec3(0.75, 0.75, 0.75)`

## 6. Tests

- [x] 6.1 Add unit tests for `make_compound` in `src/cad.rs` (0 shapes, 1 shape, 2+ shapes, type is COMPOUND)
- [x] 6.2 Add unit tests for `set_color` and `get_color` (set + get roundtrip, clamping, None default)

## 7. Build & Verify

- [x] 7.1 Run `just check` (fast compile check)
- [x] 7.2 Run `cargo test --bin rojcad` (90 tests pass)
- [x] 7.3 Run `just lint` (clippy clean)
- [x] 7.4 Run `just fmt` (formatting)
