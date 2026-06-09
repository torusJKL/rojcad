## Context

`write-step` currently maps directly to opencascade-rs `Shape::write_step(path)`, which creates a `STEPControl_Writer`, transfers one shape, writes, and drops the writer. This is a 1:1 mapping: one Janet call → one OCCT writer → one shape.

The opencascade-rs commit `fc6b0690` introduced `Shape::write_all_step(shapes, path)` which creates a single `STEPControl_Writer`, iterates over all shapes calling `Transfer()` on each, then calls `Write()` once. This produces a multi-root-entity STEP file — the standard way to store assemblies.

The Cargo.toml already points to `branch = "main"` which includes this commit. Only `Cargo.lock` needs updating.

## Goals / Non-Goals

**Goals:**
- `write-step` accepts 0+ shapes: `(write-step path & shapes)`
- With no shape args, exports all currently visible shapes from the registry
- Uses `Shape::write_all_step` internally for any count of shapes
- Single-shape calls produce identical output to current behavior
- Error semantics preserved: failures signal Janet errors

**Non-Goals:**
- No change to `write-stl` (STL is single-mesh format)
- No STEP writer configuration (schema version, units, etc.)
- No support for writing into compound shapes as root entities (each shape remains a separate root in the STEP tree)

## Decisions

### 1. Argument order: `(write-step path shape & shapes)` over `(write-step shape path ...)`

**Chosen**: `path` first, then variadic shapes.

This matches the natural language "write to path these shapes" and mirrors scp/rsync conventions. Having the path first also makes it unambiguous where the variadic portion begins. Breaking change accepted.

**Alternative considered**: Keep old order, make shape variadic at the end: `(write-step shape & shapes path)`. Rejected because variadic rest args must be last in Janet; `path` would need keyword treatment which is awkward.

**Alternative considered**: `(write-step shapes path)` where `shapes` is a tuple. Rejected because it's less ergonomic for the common single-shape case and inconsistent with the rest of the API.

### 2. Implementation: replace `rust_write_step` with `rust_write_all_step`

**Chosen**: One FFI function that always takes an array of shape pointers.

Even for single-shape calls, we construct a 1-element array and call `write_all_step`. The overhead is negligible and eliminates code duplication. The C bridge allocates a stack-allocated array of void pointers (up to a reasonable max).

**Alternative considered**: Keep `rust_write_step` for single-shape fast path. Rejected — unnecessary maintenance burden for no measurable benefit.

### 3. Zero shapes = export all visible

When `write-step` is called with only a path (no shape args), it queries the `ShapeRegistry` for all visible shape IDs via `rust_get_registered_shape_ids(1)`, resolves each to its `ShapeData` pointer via `rust_get_shape_pointer(id)`, and passes the resulting array to `rust_write_all_step`. This reuses the existing infrastructure (`SHAPE_PTR_MAP` and registry visibility tracking) without adding any new Rust code.

**Alternative considered**: A separate `write-step-visible` function. Rejected — the variadic API naturally handles this with just the path arg, and the intent is obvious.

### 4. Max shapes limit at C bridge layer

Stack-allocate the shape pointer array with a 256-element limit. This matches the existing pattern in `make-compound` (which uses 64). 256 is generous for STEP export; exceeding it would require heap allocation which isn't justified for this use case.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| **Breaking change**: existing code using old arg order silently breaks | The old order `(shape path)` will fail at runtime with a type error (path is string, shape is abstract). Explicitly called out in changelog. |
| **STEP file compatibility**: some downstream systems handle multi-root STEP files differently | Multi-root is standard STEP (AP214/AP203). If single-root is needed, pre-compose with `make-compound`. |
| **OCCT writer reuse**: `Transfer()` statefulness across calls | `STEPControl_Writer` handles this in its model; each `Transfer` adds a root entity to the same in-memory STEP model before the final `Write`. |
