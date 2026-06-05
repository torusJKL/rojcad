## Context

The project is a OCCT-based CAD REPL (`rojcad`) with a Janet language TCP server frontend and a wgpu viewer backend. STEP and STL **export** already exists (`write-step`, `write-stl`) — implemented via opencascade-rs's `Shape::write_step` and `Shape::write_stl`. The C bridge (`bridge/bridge.c`) registers the Janet functions and calls into Rust FFI (`src/main.rs`), which delegates to `src/cad.rs`. The shape type (`ShapeData`) wraps `opencascade::primitives::Shape` with metadata.

`opencascade-rs` already exposes `Shape::read_step(path)`, so STEP import is straightforward with no new dependencies.

## Goals / Non-Goals

**Goals:**
- `(read-step "path/to/file.step")` returns a rojcad shape for any valid STEP file
- Returned shapes are indistinguishable from programmatically-created shapes — they support `show`, `hide`, `shape-type`, boolean ops, `write-step`, `write-stl`, etc.
- Sensible error messages for file-not-found, invalid format, and read failures
- Round-trip test: write a shape to STEP, read it back, verify type/ID

**Non-Goals:**
- IGES import/export (out of scope — opencascade-rs already has `read_iges`/`write_iges` but no Janet bindings)
- Batch/multi-file import
- Automatic tessellation on import (caller uses `:eager` or `show` to trigger it)
- STL import (not supported)

## Decisions

1. **STEP import uses `Shape::read_step` directly** — already available in opencascade-rs. No new dependencies. Follows the same pattern as `write_step`.

2. **Error handling**: Use the same `Result<(), String>` pattern as existing export functions in `cad.rs`. C bridge panics (via `janet_panic`) on failure, mirroring `write-step`/`write-stl`.

3. **Tessellation**: Imported shapes are NOT automatically tessellated — matching the convention of other `make_*` functions. Caller opts in with `:eager` or `show`.

4. **FFI pattern**: Identical to existing export — `src/cad.rs` → `src/main.rs` (FFI) → `src/bridge.rs` (extern) → `bridge/bridge.c` (Janet C function). No architectural changes needed.

## Risks / Trade-offs

- **[STEP file variations]** — STEP AP203 vs AP214, units, coordinate systems. OCCT handles these transparently, but edge cases may appear. *Mitigation*: Start with happy-path tests; add edge-case tests as issues surface.
