## 1. Rust CAD functions — cad.rs

- [x] 1.1 Update `make_box` to support keyword params (`:w`, `:d`, `:h`, `:c`, `:pl`, `:ph`) and cube mode (single arg)
- [x] 1.2 Update `make_sphere` to support keyword params (`:r`, `:c`, `:a`)
- [x] 1.3 Add `make_cylinder` with keyword params (`:r`, `:h`, `:c`, `:dir`, `:fp`, `:tp`)
- [x] 1.4 Add `make_cone` with keyword params (`:br`, `:tr`, `:h`, `:c`, `:a`)
- [x] 1.5 Add `make_torus` with keyword params (`:rr`, `:tr`, `:c`, `:a`, `:as`, `:ae`, `:dir`)
- [x] 1.6 Add helper for validating vector params (center, direction, from-point, to-point)

## 2. FFI bridge — main.rs

- [x] 2.1 Update `rust_init_box` for new keyword-based params
- [x] 2.2 Update `rust_init_sphere` for new keyword-based params
- [x] 2.3 Add `rust_init_cylinder`
- [x] 2.4 Add `rust_init_cone`
- [x] 2.5 Add `rust_init_torus`

## 3. C bridge declarations — bridge.rs

- [x] 3.1 Add extern declarations for `rust_init_cylinder`, `rust_init_cone`, `rust_init_torus`
- [x] 3.2 Add extern declarations for `rust_shape_type` (if missing), `rust_shape_type_string`

## 4. C bridge — bridge.c

- [x] 4.1 Add `parse_vector_keyword` helper for parsing `[x y z]` arrays
- [x] 4.2 Rename `cad_make_box` to `cad_box` and update for keyword params (cube mode, corners)
- [x] 4.3 Rename `cad_make_sphere` to `cad_sphere` and update for keyword params (angle)
- [x] 4.4 Add `cad_cylinder` JANET_FN
- [x] 4.5 Add `cad_cone` JANET_FN
- [x] 4.6 Add `cad_torus` JANET_FN
- [x] 4.7 Update `cad_register_functions` registration table with new names and functions

## 5. Tests — cad.rs

- [x] 5.1 Update existing `make_box` and `make_sphere` tests for renamed `box` and `sphere`
- [x] 5.2 Add tests for `box` with keyword args, cube mode, corners
- [x] 5.3 Add tests for `sphere` with center and angle
- [x] 5.4 Add tests for `cylinder` with all constructor variants
- [x] 5.5 Add tests for `cone` (full, truncated, partial)
- [x] 5.6 Add tests for `torus` (full, partial, angled)
- [x] 5.7 Add validation tests (invalid dimensions per primitive)
