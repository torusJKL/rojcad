## Why

`BRepAlgoAPI_Cut` crashes with `SIGSEGV` when both operands are coplanar **Face** shapes (OCCT investigation confirmed `BRepAlgoAPI_Common` and `BRepAlgoAPI_Fuse` are safe — the crash only occurs in `Cut` because it requires creating a face with an inner wire/hole). Users hitting this get an unhelpful crash dump instead of a clear error. A short-term guard is needed to reject Face-Face cut with a descriptive error message, while the upstream opencascade-rs fix (`BRepBuilderAPI_MakeFace::Add`) is added.

## What Changes

- Add shape-type validation at the top of `cad::cut`
- When both inputs are `ShapeType::Face`, return an `Err` with a clear message telling the user to extrude faces to solids first
- No changes to the Janet API surface or C bridge — guard is in the Rust `cad.rs` layer
- `common` and `fuse` are NOT guarded — they work correctly with coplanar faces

## Capabilities

### New Capabilities
- `face-boolean-guard`: Validation that rejects Face-Face cut with actionable error guidance

### Modified Capabilities
*(none — no existing specs to modify)*

## Impact

- `src/cad.rs`: 1 modified function (`cut`) — gets 4-5 lines of validation at the top
- No changes to C bridge, Janet code, opencascade-rs, or build system
- Test expectations: existing tests that pass non-face shapes are unaffected; new tests verify the guard fires only for cut
