## 1. Rust: Core STL Export

- [x] 1.1 Add `write_all_stl(shapes: &[&ShapeData], path: &str)` to `src/cad.rs` — mirrors `write_all_step` but uses `Compound::from_shapes` + `Shape::from_compound` + `Shape::write_stl`
- [x] 1.2 Add unit tests for `write_all_stl` in `src/cad.rs`: single shape, multiple shapes, empty slice error

## 2. Rust: FFI Layer

- [x] 2.1 Replace `rust_write_stl` extern decl in `src/bridge.rs` with `rust_write_all_stl(shapes: *mut *mut c_void, num_shapes: c_int, path: *const c_char) -> c_int`
- [x] 2.2 Replace `rust_write_stl` function in `src/main.rs` with `rust_write_all_stl` — patterned after `rust_write_all_step`, calling `cad::write_all_stl`

## 3. C Bridge

- [x] 3.1 Replace `extern int rust_write_stl(...)` with `extern int rust_write_all_stl(...)` in `bridge/bridge.c`
- [x] 3.2 Rewrite `cad_write_stl` JANET_FN in `bridge/bridge.c` to match `cad_write_step` pattern: `janet_arity(argc, 1, -1)`, path first, optional shapes, all-visible fallback, call `rust_write_all_stl`, add proper docstring

## 4. Janet Layer

- [x] 4.1 Update `write-stl` wrapper in `boot.janet` from `[shape path]` to `[path & shapes]` with `(apply _write-stl path shapes)`
- [x] 4.2 Update `write-stl` docstring in `boot.janet` to match new signature and all-visible behavior

## 5. Tests

- [x] 5.1 Update integration test in `tests/test-variadic.sh` line 356: change `(write-stl (box 10) 123)` to `(write-stl 123 (box 10))`

## 6. Verify

- [x] 6.1 Build: `just build`
- [x] 6.2 Lint: `just lint`
- [x] 6.3 Unit tests: `just test-unit` (all 94 pass)
- [x] 6.4 REPL integration tests: `just test-repl` (all 70 pass)
