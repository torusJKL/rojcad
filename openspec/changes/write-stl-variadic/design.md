## Context

`write-stl` currently requires exactly two arguments: `(write-stl shape path)`. This differs from `write-step` which takes `(write-step path & shapes)` with optional variadic shapes and an all-visible fallback. The old `rust_write_stl` FFI accepts a single `void*` pointer; there is no multi-shape STL writer in the Rust layer.

`write-step` uses `Shape::write_all_step` from opencascade-rs which internally uses a single `STEPControl_Writer` to write multiple shapes into one file.

For STL, opencascade-rs provides only `Shape::write_stl()` (single shape), but `Compound::from_shapes()` exists to combine multiple shapes into one compound, which can then be written as a single STL file.

## Goals / Non-Goals

**Goals:**
- Make `(write-stl path & shapes)` match `write-step`'s calling convention
- Support "all visible shapes" export when no shapes given
- Combine multiple shapes into a single compound before STL export

**Non-Goals:**
- Adding STL triangulation tolerance control (not changing the 0.001 default)
- Supporting per-file-per-shape export (all shapes go into one STL file)
- Adding IGES or other format export (only STL is affected)

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| How to write multiple shapes to STL | Compound via `Compound::from_shapes` | opencascade-rs already provides this; avoids new FFI. STL cannot represent multiple root entities like STEP, so compound is the correct OCCT approach |
| Remove old `rust_write_stl` | Yes, remove entirely | Fully subsumed by `rust_write_all_stl`; keeping dead code is tech debt |
| C docstring | Provide full docstring inline (like `cad_write_step`) | Currently empty `""`; the Janet `defmeta` adds docs but bridge entry is sparse |
| Breaking change handling | Accept the break; old code using `(write-stl shape path)` must be updated | Migration is mechanical — swap first two args |

**Alternatives considered:**
- *Keep old `rust_write_stl` alongside `rust_write_all_stl`* — rejected because the old function is unused after the change and adds maintenance surface
- *Extend `rust_write_stl` in place with a `num_shapes` parameter* — rejected because it changes the C ABI anyway, so a clean rename is clearer

## Risks / Trade-offs

- **Breaking change** → Old Janet scripts with `(write-stl shape path)` will fail. Mitigation: error message is clear (type mismatch), but user education needed
- **Multi-shape STL via compound merges shapes at the OCCT level** → Some downstream tools may not handle compound STL well. Mitigation: STL is a mesh format; compounds are standard OCCT practice and most tools handle them
- **Single-shape path creates a compound unnecessarily** → Negligible cost; `Compound::from_shapes` with one element produces a single-element compound that writes identically
