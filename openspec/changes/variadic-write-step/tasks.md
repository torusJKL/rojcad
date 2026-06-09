## 1. Dependency Update

- [x] 1.1 Run `cargo update -p opencascade` to pull commit `fc6b0690` into `Cargo.lock`

## 2. Rust Core: cad.rs

- [x] 2.1 Add `write_all_step(shapes: &[&ShapeData], path: &str) -> Result<(), String>` that iterates shapes, collects `&Shape` refs, calls `Shape::write_all_step(&refs, path)`
- [x] 2.2 Remove old `write_step` (replaced by `write_all_step`), update tests to use `write_all_step`

## 3. Rust FFI: main.rs + bridge.rs

- [x] 3.1 Replace `rust_write_step(data, path)` with `rust_write_all_step(shapes: *mut *mut c_void, num_shapes: c_int, path: *const c_char) -> c_int` that builds a `Vec<&ShapeData>` from raw pointers, calls `cad::write_all_step`, returns 0/1
- [x] 3.2 Update the `extern "C"` declaration in `bridge.rs` to match new signature

## 4. C Bridge: bridge.c

- [x] 4.1 Change `extern int rust_write_step(...)` to `extern int rust_write_all_step(void **shapes, int num_shapes, const char *path)`
- [x] 4.2 Rewrite `cad_write_step`: `janet_arity(argc, 1, -1)`, extract path from `argv[0]`, with zero shape args fall back to listing visible shapes from registry via `rust_get_registered_shape_ids(1)` + `rust_get_shape_pointer`

## 5. Janet Wrapper: boot.janet

- [x] 5.1 Update the `wrap-c-fn` line: `[path & shapes]` → `(apply _write-step path shapes)`
- [x] 5.2 Update docstring to show new signature `(write-step path & shapes)` with single, multi, and all-visible examples
- [x] 5.3 Update metadata category (no change needed — stays `"io"`)

## 6. Tests

- [x] 6.1 Add `test_write_all_step_multiple` unit test in `cad.rs` that exports 2-3 shapes via `write_all_step` and validates file exists
- [x] 6.2 Update integration tests in `test-variadic.sh` for new arg order: `'(write-step 123 (box 10))'` and `'(write-step false (box 10))'`

## 7. Build & Verify

- [x] 7.1 Run `cargo check` to verify compilation
- [x] 7.2 Run `cargo test` to verify unit tests pass
- [x] 7.3 Run REPL integration tests to verify they pass
- [x] 7.4 Run `cargo clippy -- -D warnings` to verify no warnings
