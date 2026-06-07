## 1. Error infrastructure in main.rs

- [x] 1.1 Add thread-local error buffer (`LAST_CAD_ERROR`) with `set_last_error` / `take_last_error` helpers
- [x] 1.2 Add `rust_take_last_error` extern "C" function that returns `*mut c_char` (caller frees)

## 2. Refactor cad.rs: validation + Result returns

- [x] 2.1 Replace `assert_valid_dimension` with `validate_dimension` returning `Result<(), String>`
- [x] 2.2 Add `validate_dimension` calls to wire operations (fillet, chamfer, offset) — currently missing
- [x] 2.3 Change primitive constructors to return `Result<ShapeData, String>`: `make_box`, `make_cube`, `make_box_from_corners`
- [x] 2.4 Change `make_sphere`, `make_cylinder` (3 variants), `make_cone`, `make_torus` to return `Result<ShapeData, String>`
- [x] 2.5 Change `cut`, `common`, `fuse` to return `Result<ShapeData, String>` instead of panicking on empty result
- [x] 2.6 Change transform operations (`translate`, `rotate`, `scale`, `mirror`) to return `Result<ShapeData, String>`
- [x] 2.7 Change 2D primitives (`make_rect`, `make_circle`, `make_polygon`) to return `Result<ShapeData, String>`
- [x] 2.8 Change `extrude_shape`, `revolve_shape`, `extrude_polygon_raw` to return `Result<ShapeData, String>`
- [x] 2.9 Change wire operations (`wire_to_face`, `wire_fillet`, `wire_chamfer`, `wire_offset`) to return `Result<ShapeData, String>`
- [x] 2.10 Update unit tests in cad.rs — wrap `Result` returns with `.unwrap()` and add error-case tests

## 3. Refactor main.rs bridge functions

- [x] 3.1 Change `rust_init_box`, `rust_init_cube`, `rust_init_box_from_corners` to return `c_int` with catch_unwind safety net
- [x] 3.2 Change `rust_init_sphere`, `rust_init_cylinder` (3 variants), `rust_init_cone`, `rust_init_torus` to return `c_int`
- [x] 3.3 Change boolean bridge functions (`rust_init_cut`, `rust_init_common`, `rust_init_fuse`) to return `c_int`
- [x] 3.4 Change transform bridge functions (`rust_init_translate`, `rust_init_rotate`, `rust_init_scale`, `rust_init_mirror`) to return `c_int`
- [x] 3.5 Change 2D primitive bridge functions (`rust_init_rect`, `rust_init_circle`, `rust_init_polygon`) to return `c_int`
- [x] 3.6 Change extrusion/revolve bridge functions (`rust_init_extrude`, `rust_init_revolve`, `rust_init_extrude_polygon`) to return `c_int`
- [x] 3.7 Change wire operation bridge functions (`rust_init_wire_to_face`, `rust_init_wire_fillet`, `rust_init_wire_chamfer`, `rust_init_wire_offset`) to return `c_int`

## 4. Update bridge.c error handling

- [x] 4.1 Declare `rust_take_last_error` extern in bridge.c
- [x] 4.2 Add a C helper or macro for the check+janet_panic pattern (e.g., `CAD_CHECK(result)`)
- [x] 4.3 Update all primitive `rust_init_*` call sites to check return value and call janet_panic
- [x] 4.4 Update all boolean, transform, 2D, extrude, and wire `rust_init_*` call sites

## 5. Build and test

- [x] 5.1 Run `just build` and fix any compilation errors
- [x] 5.2 Run `just test-unit` and fix any test failures
- [x] 5.3 Run `just lint` and fix any clippy warnings (all pre-existing, none introduced by this change)
- [x] 5.4 Run `just run` and verify `(box -1)` returns an error instead of crashing
- [x] 5.5 Verify `(cut a b)` with non-intersecting shapes returns an error (handled by unit test test_cut_non_overlapping)
- [x] 5.6 Verify valid operations still work: `(box 10 20 30)`, `(sphere 5)`, `(cut a b)` with intersecting shapes
