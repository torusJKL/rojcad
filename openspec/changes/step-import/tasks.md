## 1. Rust core

- [x] 1.1 Add `cad::read_step(path)` in `src/cad.rs` using `Shape::read_step`
- [x] 1.2 Add unit tests for `read_step` (valid file, file-not-found, round-trip)

## 2. FFI bridge

- [x] 2.1 Add `rust_init_read_step` extern declaration in `src/bridge.rs`
- [x] 2.2 Add `rust_init_read_step` FFI function in `src/main.rs`

## 3. Janet C bridge

- [x] 3.1 Add `rust_init_read_step` extern declaration in `bridge/bridge.c`
- [x] 3.2 Add `cad_read_step` JANET_FN in `bridge/bridge.c` with docstring and usage example
- [x] 3.3 Register `read-step` in the Janet function table in `bridge/bridge.c`

## 5. Verification

- [x] 5.1 Run `just check` to verify compilation
- [x] 5.2 Run `just test-unit` to verify all unit tests pass
- [x] 5.3 Run `just lint` to verify clippy passes (all errors are pre-existing, zero new warnings from this change)
