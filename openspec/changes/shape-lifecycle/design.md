## Context

Currently every shape is registered in the viewer at creation time — `ShapeData::new()` calls `global_shape_registry().register()` unconditionally. There is no way to create a shape purely for computation. When a ShapeData is collected by Janet's GC, the registry entry leaks (no `Drop` impl). The viewer also has dead channel infrastructure (`ReplToViewer` messages) that was never connected.

## Goals / Non-Goals

**Goals:**
- Decouple shape creation from viewer registration — shapes exist in Janet first, viewer second
- Deferred tessellation by default; eager opt-in via `:eager` keyword
- Explicit display lifecycle: `show` registers, `hide` hides, GC cleans up
- `purge` function for immediate viewer removal
- `display` helper for `show` + return in one step
- Automatic registry cleanup when ShapeData is GC'd (implement `Drop`)
- Remove dead `ReplToViewer` channel infrastructure

**Non-Goals:**
- Changing the OCCT shape creation API
- Persistence or serialization of shape state
- Undo/redo of display state
- Batch operations on the registry

## Decisions

### D1: ShapeData holds tessellated data; registration is separate

`ShapeData` gains `mesh: Option<MeshData>`, `edge_polylines: Vec<Vec<[f64; 3]>>`, and `registered: bool` fields. Creation functions (`make_box`, etc.) write the OCCT shape into `ShapeData` but conditionally tessellate based on `:eager`. The viewer registry only receives data when `show` is called.

### D2: `:eager` keyword on all shape-creating functions

Every `JANET_FN` that creates a shape (box, sphere, cylinder, cone, torus, cut, common, fuse, translate, rotate, scale, mirror) accepts `:eager`. When set, the Rust-level creation function tessellates the shape immediately (calling `extract_mesh` + `extract_edge_polylines`) and populates the `ShapeData` fields. When not set, mesh remains `None` and tessellation is deferred to the first `show`.

The `:eager` flag is threaded through the C bridge as a keyword, parsed by a shared helper, and passed to each `rust_init_*` function.

**Why not just always tessellate?** Most shapes are intermediates in compound expressions and are never displayed. Tessellation is expensive (OCCT mesh extraction + Rust data conversion). Deferring saves CPU time and memory for the common case of `(fuse (box 10) (sphere 5))`.

**Why not automatic lazy tessellation (tessellate on first `show`)?** That's the default behavior when `:eager` is not specified. `:eager` is an optimization hint for shapes that are known to be displayed.

### D3: `show` is the sole path into the viewer

`show(shape_data)`:
1. If `shape_data.mesh` is `None`: tessellate, store in `shape_data`
2. If not `registered`: call `global_shape_registry().register(entry)`, set `registered = true`
3. Call `global_shape_registry().set_visible(shape_id, true)`

`show` is idempotent — calling it on an already-visible shape is a no-op (just flips the flag again).

### D4: `hide` only toggles visibility

`hide(shape_data)`:
1. If `registered`: call `global_shape_registry().set_visible(shape_id, false)`

Does not remove from registry. Does not free tessellation data. `show` can restore visibility without re-tessellating.

### D5: `Drop for ShapeData` removes from registry

```rust
impl Drop for ShapeData {
    fn drop(&mut self) {
        if self.registered {
            global_shape_registry().remove(self.shape_id);
        }
    }
}
```

This is called by `rust_shape_drop` when Janet's GC collects the abstract value. The viewer sees the generation counter bump and removes the shape on the next frame.

### D6: `purge` is a C function (not a macro)

`purge` is registered as a C JANET_FN in bridge.c:
1. Calls `global_shape_registry().remove(shape_id)` — immediate viewer update
2. Marks the ShapeData with `purged = true` so subsequent operations fail fast

A macro-based approach (`(purge b)` expanding to `(do (registry-remove b) (def b nil))`) was the original intent, but `defmacro` is not available in Janet bootstrap mode (`JANET_BOOTSTRAP=1`). The `:macro` attribute on `def` works, but the macro expander only passes user arguments (not `&form`/`&env`), and more critically, any `def` inside a `do` block is not at the top-level scope, so the binding name never reaches `core-env` and isn't visible to subsequent REPL commands.

Instead, `purge` is a C function that takes the evaluated shape value and removes it from the registry. Users unbind the variable separately: `(purge b) (def b nil)`.

### D7: `display` is a C function (not a macro)

`display` is registered as a C JANET_FN that calls `show` on the shape and returns it:
```janet
(def b (display (box 10)))   ;; create, show, bind in one step
```

A `(display b (box 10))` macro was the original design, but it ran into fundamental constraints of the bootstrapped Janet environment:

1. **`defmacro` is unavailable** in `JANET_BOOTSTRAP=1` mode. The `:macro` attribute on `def` works, but the macro expander passes only user arguments (not `&form`/`&env`).
2. **`core-env` is a compile-time snapshot** captured at boot.janet line 10. Only Rust-registered C functions and boot-time definitions before line 10 are visible to the REPL compiler.
3. **Scope issues with macro expansion**: A `(display b (box 10))` macro would expand to a `do` block containing `def`. Inside a `do`, the `def` is NOT at the top-level scope, so `defleaf` (the Janet compiler's handler for `def`) does NOT add `b` to `core-env`. This means `b` is invisible to subsequent REPL commands.
4. **Recursive expansion**: A macro expanding to `(def b (display (box 10)))` references `display` in its own output, causing infinite recursive macro expansion because the macro name shadows the C function in the env.

The C function approach `(def b (display (box 10)))` avoids all of these issues because `def` is at the top-level scope of the compiled form, so `b` is added to `core-env` at compile time and visible across REPL connections.

### D8: `:eager` is a single-keyword flag, not a key-value pair

Parsed as a boolean keyword presence: `(box 10 :eager)` — if `:eager` appears in the arg list, it's true. No `:eager true` or `:eager false`. This matches the style of other boolean flags in the existing codebase.

### D9: Remove the dead `ReplToViewer` channel

The `ReplToViewer` enum (`UpdateShapes`, `RemoveShape`, `ClearAll`) and the `mpsc` channel pair in `spawn_viewer()` are removed. The viewer has always communicated via the global `ShapeRegistry` singleton; the channel was scaffolding that was never connected. Removing it simplifies the threading model and eliminates dead code.

`ViewerToRepl` (selection events) is retained — it's actively used.

## Risks / Trade-offs

- **GC timing gap**: After `(def b nil)`, the ShapeData remains alive until the next GC cycle. The shape stays visible during that window. **Accepted** — this is standard Janet behavior. Users who want immediate cleanup use `(purge b)` which removes from the viewer synchronously.
- **Tessellation cost on first `show`**: If a large model is `def`'d without `:eager`, the first `show` will block while tessellating. **Mitigation** — the `:eager` keyword exists for exactly this scenario, and the blocking is a one-time cost.
- **Backwards incompatibility**: Existing scripts that rely on immediate display will break. **Mitigation** — `display` macro provides a direct migration path, and the project is pre-1.0 with no known external consumers.
- **ShapeData size increase**: Adding `mesh`, `edges`, and `purged` fields increases memory per ShapeData, even for never-displayed intermediates. **Accepted** — `None` meshes are small, and intermediates are short-lived.
