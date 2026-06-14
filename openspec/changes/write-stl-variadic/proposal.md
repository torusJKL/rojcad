## Why

`write-stl` is the only export function with a different argument order and no support for variadic shapes or all-visible fallback. This inconsistency forces users to remember two different calling conventions and prevents quick "export everything" workflows with STL.

## What Changes

- **BREAKING**: `write-stl` signature changes from `(write-stl shape path)` to `(write-stl path & shapes)` — matches `write-step` convention
- `write-stl` gains variadic shape support: export one or more shapes as a compound into a single STL file
- `write-stl` with no shape arguments exports all currently visible shapes (matching `write-step` behavior)
- Old single-shape `write_stl` Rust function and `rust_write_stl` FFI entry removed (subsumed by `write_all_stl`)

## Capabilities

### New Capabilities

*(none — this modifies an existing capability)*

### Modified Capabilities

- `cad-export`: `write-stl` requirement changes from fixed 2-arg `(write-stl shape path)` to variadic `(write-stl path & shapes)` with all-visible fallback

## Impact

- `src/cad.rs`: Replace `write_stl(&ShapeData, &str)` with `write_all_stl(&[&ShapeData], &str)` using `Compound::from_shapes`
- `src/bridge.rs`: Replace `rust_write_stl` extern decl with `rust_write_all_stl`
- `src/main.rs`: Replace `rust_write_stl` FFI function with `rust_write_all_stl`
- `bridge/bridge.c`: Rewrite `cad_write_stl` JANET_FN to match `cad_write_step` pattern; update extern decl
- `boot.janet`: Update wrapper and docstring
- `tests/test-variadic.sh`: Update error test for new argument order
- `openspec/specs/cad-export/spec.md`: Update `write-stl` requirement spec
