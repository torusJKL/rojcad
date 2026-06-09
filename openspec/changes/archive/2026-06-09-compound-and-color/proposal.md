## Why

rojcad has no way to group multiple shapes into a single logical assembly, and no way to assign colors to shapes. The `ShapeData`/`ShapeEntry` types already carry a `color: Option<[f64;3]>` field, but it's a dead placeholder — the viewer ignores it and everything renders in hardcoded grey. These two gaps compound (no pun): without grouping, complex models can't be organized; without color, they can't be visually distinguished.

## What Changes

- Add `compound` — a Janet function that wraps multiple shapes into a single OCCT `Compound` (topological container, no boolean overhead)
- Add per-shape color API: `color` (setter, mutates in place), `get-color` (getter), and a `:color` keyword on `compound`
- Wire the `color` field through to the viewer so each shape renders with its assigned color instead of hardcoded grey
- Selection highlight remains blue (overrides own color)

## Capabilities

### New Capabilities
- `compound-group`: Group 1+ shapes into an OCCT Compound via `compound`, with an optional `:color` keyword
- `shape-color`: Per-shape color API — `color` to set (mutates in place, returns same shape), `get-color` to read back; viewer renders per-mesh color

### Modified Capabilities
(none)

## Impact

- `src/cad.rs`: New `make_compound()` and `set_color()` Rust functions
- `bridge/bridge.c`: New `_cad_comp` and `_cad_set_color` Janet-callable C functions
- `src/bridge.rs`: New `extern "C"` declarations
- `src/main.rs`: New FFI bridge functions
- `boot.janet`: New `compound`, `color`, `get-color` wrapper functions; `:color` keyword on `compound`
- `src/viewer/app.rs`: Per-mesh color in `CadMesh`; `SurfaceDrawer` passes color to GPU
- `src/viewer/shader.wgsl`: Fragment shader reads per-mesh color uniform instead of hardcoded grey
