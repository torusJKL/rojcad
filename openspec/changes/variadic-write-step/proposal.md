## Why

`write-step` currently exports only a single shape. opencascade-rs has gained `Shape::write_all_step` (commit `fc6b0690`) which can transfer multiple shapes to one STEP writer before writing. We should extend `write-step` to accept any number of shapes so users can export all visible geometry in one call.

## What Changes

- **`write-step` argument order changes** — **BREAKING**: from `(write-step shape path)` to `(write-step path shape & shapes)`
- `write-step` now accepts one or more shapes after the path
- `write-stl` is unchanged (STL format inherently stores a single mesh)

## Capabilities

### New Capabilities

(none — this modifies an existing capability)

### Modified Capabilities

- `cad-export`: `write-step` requirement changed from exactly one shape to one-or-more shapes. Argument order is reversed. Behavior for single-shape calls is preserved (same output file format, same error semantics).

## Impact

- **API**: `(write-step shape path)` → `(write-step path shape & shapes)` — all callers must update
- **Dependency**: opencascade-rs must be at `fc6b0690` (already on `main` branch, just needs `cargo update`)
- **Bridge**: C `cad_write_step` rewired to construct a shape pointer array and call new `rust_write_all_step`
- **Rust**: New `cad::write_all_step` function using `Shape::write_all_step`; old single-shape FFI replaced
- **Tests**: Unit test for multi-shape export; integration tests updated for new arg order
- **Docstring**: Updated to reflect new signature and usage examples
