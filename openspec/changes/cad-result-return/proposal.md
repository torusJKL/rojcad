## Why

Every CAD operation (`box`, `sphere`, `cut`, etc.) currently panics on invalid input (negative dimensions, empty boolean results). Because these panics cross the `extern "C"` FFI boundary, they abort the entire process instead of returning a recoverable error to the Janet REPL. This makes the application brittle — a single typo from a user kills the connection.

## What Changes

- Replace `assert_valid_dimension` (which panics) with `validate_dimension` (which returns `Result`)
- Change all CAD functions in `cad.rs` to return `Result<ShapeData, String>` instead of panicking on validation errors and empty boolean results
- Change all 27 `extern "C"` bridge functions in `main.rs` to return a `c_int` error code instead of panicking in the `catch_unwind` error handler
- Update `bridge.c` to check the error code and call `janet_panic` to surface the error to Janet
- The 4 wire operations that lack `catch_unwind` entirely get proper error handling via the same `Result`-based approach
- Remove the `catch_unwind` pattern from bridge functions for predictable errors (OCCT library panics for truly unexpected failures can remain wrapped)

## Capabilities

### New Capabilities
- `cad-error-propagation`: CAD operations validate inputs and surface errors to Janet callers via `janet_panic` instead of aborting the process

### Modified Capabilities
- (none — `repl-error-recovery` already requires catching all evaluation errors; this change fulfills that requirement rather than modifying it)

## Impact

- `src/cad.rs`: All public CAD functions change return types from `ShapeData` / `()` to `Result<ShapeData, String>` / `Result<(), String>`
- `src/main.rs`: All 27 `rust_init_*` functions change from `void` to `c_int` return type; `catch_unwind` error handlers return error codes instead of panicking
- `bridge/bridge.c`: All ~27 `rust_init_*` call sites gain a return-value check wrapping `janet_panic`
- `src/types.rs`: No changes expected
- Tests: CAD unit tests need `unwrap()` or `?` on `Result`-returning functions; new tests for error cases should be added
