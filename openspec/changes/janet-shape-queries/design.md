## Context

Janet functions currently have no way to query which shapes are registered in the viewer's `ShapeRegistry` or which shapes are currently selected in the 3D viewer. The `poll-selection` mechanism provides one-shot events (toggle-on/off/clear) via atomics, but there is no durable state query. The `ShapeRegistry` stores `ShapeEntry` values (mesh, edges, visibility, color) indexed by `ShapeId`, but there is no reverse mapping from `ShapeId` to the Janet GC-allocated `ShapeData` abstract value.

The viewer thread owns `selected_ids: HashSet<ShapeId>` in `ViewerState` and reports selection changes via:
- `LAST_SELECTION` / `LAST_SELECTION_ACTION` atomics (consumed by `poll-selection`)
- `ViewerToRepl::SelectionChanged` mpsc message (receiver is currently discarded)

Neither path allows enumerating the full selection set from Janet.

## Goals / Non-Goals

**Goals:**
- `(selected-shapes)` returns a tuple of `ShapeData` abstract values currently selected in the viewer
- `(list-shapes &keys :visible :hidden)` returns a tuple of all registered `ShapeData` values, filterable by visibility
- Returned shape objects are the actual Janet GC-managed values (not copies or IDs), passable to `hide`, `show`, `visible?`, `shape-type`, etc.
- Minimal per-shape overhead — each shape constructor adds one pointer-registration call
- Thread-safe: viewer thread writes selection state, REPL thread reads it

**Non-Goals:**
- A `(get-shape id)` function for ID-to-object lookup (pointer map is internal infrastructure, not a public API)
- Changing `poll-selection` or `on-select` API
- Selection state persistence or undo
- Batch selection operations from Janet (set, clear, toggle) — query only

## Decisions

### D1: Global `SHAPE_PTR_MAP` for `ShapeId → ShapeData*` lookup

A static `OnceLock<RwLock<HashMap<ShapeId, *mut c_void>>>` maps each shape's ID to its Janet GC-allocated abstract memory. This is the bridge between the integer-based `ShapeRegistry` and the pointer-based Janet world.

```
     ShapeData (Janet GC heap)         ShapeRegistry (shared)
     ┌──────────────────┐              ┌──────────────────────┐
     │ shape_id: 42     │              │ 42: ShapeEntry       │
     │ shape: OCCT obj  │              │     visible: true    │
     │ mesh: ...        │              │     selected: false  │
     │ registered: true  │              │     mesh: ...        │
     └──────┬───────────┘              └──────────────────────┘
            │         ▲                         ▲
            │         │                         │
            ▼         │                         │
     ┌─────────────────┴──────────┐              │
     │  SHAPE_PTR_MAP             │              │
     │  { 42: ptr-to-ShapeData }  ├──────────────┘
     └────────────────────────────┘  lookup by ID
```

**Registration**: Called in `rust_init_box`, `rust_init_sphere`, etc. — right after `ptr::write(dest, shape_data)`. The pointer is the `dest` parameter (the `janet_abstract`-allocated memory).

```rust
// In main.rs, after ptr::write in each rust_init_*:
global_shape_ptr_map().register(shape_id, dest);
```

**Unregistration**: In `ShapeData::drop`:
```rust
impl Drop for ShapeData {
    fn drop(&mut self) {
        global_shape_ptr_map().unregister(self.shape_id);
        if self.registered {
            global_shape_registry().remove(self.shape_id);
        }
    }
}
```

**Alternatives considered:**
- Thread-local `ShapeData` cache in the bridge: Breaks for cross-fiber access.
- No pointer map, return IDs only: Requires users to manually track shape objects. Rejected because the goal is direct operability.

### D2: `SELECTED_IDS` global for selection state

A `OnceLock<RwLock<HashSet<ShapeId>>>` stores the current selection set. Written by the viewer thread in `handle_click`, read by the REPL thread when `selected-shapes` is called.

```rust
// types.rs
pub static SELECTED_IDS: OnceLock<RwLock<HashSet<ShapeId>>> = OnceLock::new();
```

```rust
// app.rs — handle_click, after modifying state.selected_ids:
*SELECTED_IDS.get().unwrap().write().unwrap() = state.selected_ids.clone();
```

**Alternatives considered:**
- Add `selected: bool` to `ShapeEntry` in `ShapeRegistry`: Ties selection state to the rendering registry. Creates coupling between viewer selection logic and registry structure. The existing registry is for rendering data; selection is orthogonal.
- Reuse the existing `ViewerToRepl` mpsc channel: The receiver is currently discarded. Wiring it up would require a continuous polling loop on the REPL side, adding complexity for a simple query.
- Lock-free approach with generation counter + snapshot: Over-engineered for selection sets that change <10 times/second.

### D3: `ShapeRegistry::hidden_shapes()` method

Mirrors the existing `visible_shapes()`:

```rust
pub fn hidden_shapes(&self) -> Vec<ShapeEntry> {
    let map = self.inner.read().expect("poisoned");
    map.values()
        .filter(|e| !e.visible)
        .map(clone_entry)
        .collect()
}
```

### D4: C bridge patterns for the two JANET_FN handlers

**`cad_selected_shapes`**:
1. Call `rust_get_selected_shape_ids(&count)` → get array of `u64` IDs
2. For each ID, call `rust_get_shape_pointer(id)` → get `ShapeData*`
3. Build Janet tuple via `janet_tuple_n(parts, count)`
4. Free the ID array via `rust_free_u64_array`

**`cad_list_shapes`**:
1. Parse keywords: detect `:visible` or `:hidden` (mutually exclusive; `:hidden` takes precedence if both provided; neither means `:all`)
2. Call `rust_get_registered_shape_ids(filter, &count)` with filter enum: 0=all, 1=visible, 2=hidden
3. Same ID→pointer lookup and tuple construction as above
4. Free the ID array

Both functions return `nil` for any ID that has no registered `ShapeData*` (e.g., if GC collected it between query and lookup — unlikely but safe).

### D5: Shape pointer registration happens in Rust `extern "C"` init functions, not in C bridge

The `rust_init_*` functions in `main.rs` already receive `dest: *mut c_void`. Adding `global_shape_ptr_map().register(shape_id, dest)` at the end of each (after successful `ptr::write`) centralizes the registration in Rust. The C bridge only needs `extern void rust_register_abstract(void *data);` if we want a C-side fallback, but the Rust-side approach is cleaner.

Actually, simpler: just add it inside each `rust_init_*` function. No new extern declaration needed in C.

Wait — but the caller in C owns the `janet_abstract` pointer. The shape is allocated by `janet_abstract` in C, then the `dest` pointer is passed to `rust_init_*` which does `ptr::write`. So `dest` is indeed the pointer to the live ShapeData in GC memory. Registering at that point is safe.

### D6: Memory safety of the pointer map

The `*mut c_void` in `SHAPE_PTR_MAP` points to Janet GC-allocated memory. The GC guarantees:
- Memory is valid as long as any Janet value references it
- `janet_wrap_abstract(ptr)` creates a new reference, incrementing the refcount
- When the returned tuple goes out of scope in Janet, the GC may collect the wrapped ShapeData, which triggers `ShapeData::drop`, which unregisters from `SHAPE_PTR_MAP`

Key insight: **the pointer map entry is only used during the bridge call**. Once `janet_wrap_abstract` creates a Janet value from the pointer, Janet's GC owns the lifetime. The pointer map is just the lookup mechanism — it doesn't extend lifetimes.

Unregistration in `ShapeData::drop` handles the GC-collection case (when all Janet references are gone).

## Risks / Trade-offs

- **Stale pointer in map during concurrent GC**: If the REPL thread reads the map while the GC is collecting a `ShapeData` on another fiber, the pointer could be dangling. **Mitigation**: `SHAPE_PTR_MAP` is protected by `RwLock`, and unregistration happens inside `Drop` which is called by the GC during a safepoint. The bridge function reads the map, wraps the pointer via `janet_wrap_abstract` (which increments refcount and prevents collection), then returns. This is the standard Janet FFI pattern.
- **~25 lines added to bridge.c**: Each `JANET_FN` gets a `rust_register_abstract(shape);` line. Mechanical but tedious. **Mitigation**: Can be done with search-and-replace patterns.
- **Thread safety of SELECTED_IDS write in handle_click**: `handle_click` runs on the winit event loop thread. Writing a `RwLock<HashSet>` involves allocation (clone of the set). **Accepted**: Selection changes happen on user click (~10/sec max), not per-frame. The allocation is negligible.
- **Two sources of truth for selection**: `ViewerState.selected_ids` (hot path) and `SELECTED_IDS` global (query path). **Accepted**: `selected_ids` is the canonical source; `SELECTED_IDS` is a synced copy. Desync window is one `handle_click` call — acceptable.
