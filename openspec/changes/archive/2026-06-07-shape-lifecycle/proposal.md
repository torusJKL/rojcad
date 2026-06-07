## Why

Every CAD operation (box, sphere, fuse, translate, etc.) unconditionally registers the resulting shape in the viewer registry and makes it visible. This means: (1) intermediate shapes in compound expressions like `(fuse (box 10) (sphere 5))` clutter the viewer, (2) shapes that go out of scope in Janet are never cleaned up from the registry (memory leak), and (3) there is no way to create a shape purely for computation without it appearing in the viewer. This change introduces an explicit lifecycle model where shapes exist in Janet first, and are only registered in the viewer on demand.

## What Changes

- **BREAKING**: `ShapeData::new()` no longer registers shapes in the viewer registry. Registration is deferred to an explicit `show` operation.
- **BREAKING**: Shapes are no longer tessellated at creation time by default. A new `:eager` keyword on all shape-creating functions opts into immediate tessellation.
- Add `(show <shape>)` — register a shape in the viewer and make it visible. First call tessellates (if needed), registers, and sets visible. Subsequent calls just flip visibility back on.
- Add `(hide <shape>)` — set visible flag to false (stays registered).
- Add `(purge <shape-var>)` — macro that removes from registry and unbinds the Janet variable (`(def <shape-var> nil)`).
- Add `(display <name> <expr>)` — macro equivalent to `(def <name> <expr>) (show <name>)`.
- Add `Drop for ShapeData` that calls `registry.remove()` — shapes are cleaned up from the viewer when Janet's GC collects the abstract value.
- Remove dead `ReplToViewer` channel (`UpdateShapes`, `RemoveShape`, `ClearAll` messages and all associated infrastructure).

## Capabilities

### New Capabilities
- `shape-lifecycle`: Shape creation, display, and cleanup lifecycle. Covers the explicit show/hide/purge model, deferred tessellation with `:eager` opt-in, automatic cleanup on GC, and the `display` shorthand.

### Modified Capabilities

*(None — no existing specs to modify.)*

## Impact

- `src/types.rs`: Add `Drop for ShapeData`, add `mesh`/`edges`/`registered` fields to `ShapeData`, remove registry call from `ShapeData::new()`
- `src/cad.rs`: Every creation function (`make_box`, `make_cube`, `make_sphere`, `make_cylinder`, `make_cone`, `make_torus`, `cut`, `common`, `fuse`, `translate`, `rotate`, `scale`, `mirror`) — remove registry interaction, accept `:eager` flag, conditionally tessellate
- `src/main.rs`: Update `rust_shape_drop` to call `registry.remove()`, update all `rust_init_*` functions to pass `:eager` flag through
- `bridge/bridge.c`: Add `show`, `hide`, `purge` functions to C bridge, update all `cad_*` JANET_FN to accept and forward `:eager` keyword, register `display` macro
- `boot.janet`: Remove dead channel references if any
- `src/viewer/mod.rs`: Remove `ReplToViewer` enum, `repl_tx`/`repl_rx` channel creation
- `src/viewer/app.rs`: Remove `repl_rx` field from `ViewerApp`
