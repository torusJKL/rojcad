## Why

Users need to load existing 3D models from STEP files into the rojcad REPL for visualization, inspection, and further CAD operations. STEP import enables professional CAD interoperability. Export (`write-step`, `write-stl`) already exists — this change adds the import counterpart.

## What Changes

- Add `read-step` Janet function: load a STEP file from disk and return a shape
- Add Rust FFI bridge function (`rust_read_step`)
- Add Rust CAD function (`cad::read_step`) wrapping `opencascade-rs`'s `Shape::read_step`
- Unit tests for STEP import (file-not-found, valid-file round-trip)
- Update docs if applicable

No breaking changes — all existing APIs remain unchanged.

## Capabilities

### New Capabilities
- `step-import`: Read STEP (.step, .stp) files from disk into a rojcad shape, returning a shape value that can be shown, inspected, and used in boolean operations

### Modified Capabilities
<!-- No existing capabilities have requirement changes. -->

## Impact

- `src/cad.rs`: Add `read_step` function
- `src/main.rs`: Add `rust_read_step` FFI bridge function
- `src/bridge.rs`: Add extern declaration for `rust_read_step`
- `bridge/bridge.c`: Add `cad_read_step` Janet C function definition and register it
- `src/cad.rs` tests: Add `test_read_step` unit tests
