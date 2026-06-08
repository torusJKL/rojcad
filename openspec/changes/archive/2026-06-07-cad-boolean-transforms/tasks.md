## 1. Fork opencascade-rs

- [x] 1.1 Clone opencascade-rs locally at `../opencascade-rs`
- [x] 1.2 Add `translated(&self, offset: DVec3) -> Shape` method to `crates/opencascade/src/primitives/shape.rs` using `BRepBuilderAPI_Transform_new` with `copy=true` and `gp_Trsf::set_translation_vec`
- [x] 1.3 Add `rotated(&self, axis: DVec3, angle: f64) -> Shape` method using `gp_Trsf::SetRotation`
- [x] 1.4 Add `scaled(&self, point: DVec3, factor: f64) -> Shape` method using `gp_Trsf::SetScale`
- [x] 1.5 Add `mirrored(&self, origin: DVec3, dir: DVec3) -> Shape` method using `gp_Trsf::set_mirror_axis`
- [x] 1.6 Update `Cargo.toml` to point to local opencascade-rs

## 2. Add fuse (union) operation

- [x] 2.1 Add `fuse(a: &ShapeData, b: &ShapeData) -> ShapeData` to `src/cad.rs` using `Shape::union()`, following the `cut`/`common` pattern
- [x] 2.2 Add `rust_init_fuse(dest, a, b)` FFI function to `src/main.rs` with `catch_unwind`
- [x] 2.3 Add `rust_init_fuse` extern declaration to `src/bridge.rs`
- [x] 2.4 Add `cad_fuse` JANET_FN to `bridge/bridge.c` — unwrap two shapes, allocate result, call `rust_init_fuse`
- [x] 2.5 Register `cad_fuse` in the `cfuns` array in `cad_register_functions`

## 3. Add translate operation

- [x] 3.1 Add `translate(data: &ShapeData, dx: f64, dy: f64, dz: f64) -> ShapeData` to `src/cad.rs` using `Shape::translated()`
- [x] 3.2 Add `rust_init_translate(dest, data, dx, dy, dz)` FFI function to `src/main.rs`
- [x] 3.3 Add `rust_init_translate` extern declaration to `src/bridge.rs`
- [x] 3.4 Add `cad_translate` JANET_FN to `bridge/bridge.c` — parse shape + dx/dy/dz positional args and `:t` keyword, allocate result, call `rust_init_translate`
- [x] 3.5 Register `cad_translate` in the `cfuns` array

## 4. Add rotate operation

- [x] 4.1 Add `rotate(data: &ShapeData, axis: DVec3, angle: f64) -> ShapeData` to `src/cad.rs` using `Shape::rotated()`
- [x] 4.2 Add `rust_init_rotate(dest, data, ax, ay, az, angle)` FFI function to `src/main.rs`
- [x] 4.3 Add `rust_init_rotate` extern declaration to `src/bridge.rs`
- [x] 4.4 Add `cad_rotate` JANET_FN to `bridge/bridge.c` — parse shape, `:a`/`:ar` angle, and `:x`/`:y`/`:z`/`:r` axis; allocate result; call `rust_init_rotate`
- [x] 4.5 Register `cad_rotate` in the `cfuns` array

## 5. Add scale operation

- [x] 5.1 Add `scale(data: &ShapeData, factor: f64, center: DVec3) -> ShapeData` to `src/cad.rs` using `Shape::scaled()`
- [x] 5.2 Add `rust_init_scale(dest, data, factor, cx, cy, cz)` FFI function to `src/main.rs` (center pointer may be NULL)
- [x] 5.3 Add `rust_init_scale` extern declaration to `src/bridge.rs`
- [x] 5.4 Add `cad_scale` JANET_FN to `bridge/bridge.c` — parse shape + factor positional, `:o [x y z]` keyword; allocate result; call `rust_init_scale`
- [x] 5.5 Register `cad_scale` in the `cfuns` array

## 6. Add mirror operation

- [x] 6.1 Add `mirror(data: &ShapeData, origin: DVec3, dir: DVec3) -> ShapeData` to `src/cad.rs` using `Shape::mirrored()`
- [x] 6.2 Add `rust_init_mirror(dest, data, ox, oy, oz, dx, dy, dz)` FFI function to `src/main.rs`
- [x] 6.3 Add `rust_init_mirror` extern declaration to `src/bridge.rs`
- [x] 6.4 Add `cad_mirror` JANET_FN to `bridge/bridge.c` — parse shape + 6 positional coords; allocate result; call `rust_init_mirror`
- [x] 6.5 Register `cad_mirror` in the `cfuns` array

## 7. Change angle keywords from radians to degrees

- [x] 7.1 In `cad_sphere` (bridge.c): after parsing `:a`, multiply `angle *= (M_PI / 180.0)` before calling `rust_init_sphere`. Add `:ar` keyword that passes through without conversion.
- [x] 7.2 In `cad_cone` (bridge.c): same conversion for `:a`, add `:ar` passthrough.
- [x] 7.3 In `cad_torus` (bridge.c): same conversion for `:a`/`:as`/`:ae`, add `:ar`/`:asr`/`:aer` passthrough.
- [x] 7.4 Update docstrings on `cad_sphere`, `cad_cone`, `cad_torus` and all new functions: `:a` now documents as degrees, `:ar` as radians.

## 8. Write unit tests

- [x] 8.1 Add `test_fuse` — union two overlapping boxes, verify originals unchanged
- [x] 8.2 Add `test_translate` — translate a box, verify original at origin, moved at offset
- [x] 8.3 Add `test_rotate` — rotate about Z, verify shape type is SOLID
- [x] 8.4 Add `test_scale` — scale 2x about origin, verify shape type is SOLID
- [x] 8.5 Add `test_scale_with_center` — scale 2x about custom point
- [x] 8.6 Add `test_mirror` — mirror about X axis
- [x] 8.7 Add `test_immutability` — verify all transform ops leave original shape untouched (visible, type unchanged)
- [x] 8.8 Run `just test` — 45 tests pass (34 old + 11 new)

## 9. Verify

- [x] 9.1 Run `just build` — clean compilation with local opencascade-rs
- [x] 9.2 Run `cargo test` — all 45 tests pass
- [x] 9.3 Run `cargo clippy -- -D warnings` — only pre-existing errors in viewer/, no new errors from changed files
- [x] 9.4 Verify manually via REPL: create shapes, apply transforms, check results
