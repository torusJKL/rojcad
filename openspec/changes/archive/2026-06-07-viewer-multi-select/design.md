## Context

The viewer currently tracks selection as a single `selected_id: Option<ShapeId>` (`u64`) in `ViewerState` (`src/viewer/app.rs:790`). Selection is set on mouse press — there is no click-vs-drag distinction. The Janet bridge uses a single `LAST_SELECTION: AtomicU64` sentinel (`src/types.rs:22`), where `0` = no event, `u64::MAX` = deselected, and any other value = selected shape ID. No modifier keys are consulted during click handling.

The viewer renders with two WGSL fragment pipelines: `fs_main` (gray) and `fs_highlight` (blue), selected per-mesh by comparing each mesh's `shape_id` against `selected_id`. Edge instances are also split into active/inactive buffers based on this comparison.

This design enables multi-shape selection through keyboard modifiers (Shift, Ctrl) while maintaining backward compatibility with the Janet API.

## Goals / Non-Goals

**Goals:**
- Support selecting multiple shapes in the 3D viewer simultaneously
- Plain click: replace selection with the clicked shape (on mouse release)
- Shift+click: add clicked shape to selection (on release)
- Ctrl+click: toggle clicked shape in/out of selection (on release)
- Ctrl/Shift+click on empty space: no-op
- Plain click on empty space: clear selection
- Visual feedback (blue highlight + active edges) for all selected shapes
- Report selection changes to Janet via the existing `poll-selection` mechanism
- Prevent accidental selection during camera orbit (drag)

**Non-Goals:**
- Lasso / rubber-band selection
- Selection groups or named selection sets
- Programmatic multi-select from Janet (Janet can still only see selection events)
- Changing existing Janet `on-select` or `poll-selection` function signatures (backward compat)
- Changing the receiver side of the `ViewerToRepl` mpsc channel (currently discarded)

## Decisions

### D1: `HashSet<ShapeId>` for selection state

Replace `selected_id: Option<u64>` with `selected_ids: HashSet<u64>`. Choice over `BTreeSet` or `Vec`: `HashSet` gives O(1) membership checks (used per-mesh in rendering hot path: `SurfaceDrawer::render` iterates all meshes each frame). `FxHashSet` (from `rustc-hash`) is preferred for u64 keys to avoid DOS-resistant hashing overhead where it isn't needed.

**Alternatives considered:**
- `Vec<u64>`: O(n) `.contains()` on render path. Acceptable for small selection sets but penalizes large assemblies.
- `BTreeSet<u64>`: O(log n) lookups. Slower than hash for flat u64 keys.

### D2: Wait-for-release click detection

Move `handle_click` call from `MouseInput { state: Pressed }` to `MouseInput { state: Released }` with a drag-distance threshold (~3px). On press, store `click_start_pos`. On release, if Euclidean distance from start pos < threshold, treat as a click and run `handle_click`. Otherwise, it was a drag — no selection change.

This prevents accidental selection when the user intends to orbit. Camera rotation still activates on press (via `CursorMoved` checking `mouse_pressed`), so there's no startup latency for drag.

**Alternatives considered:**
- Keep on-press behavior: Simpler code but violates the requirement.
- Highlight under cursor on press, commit on release: Better UX but adds complexity of preview rendering.

### D3: Two-atomite bridge for selection events

Replace `LAST_SELECTION: AtomicU64` with a pair:

```rust
pub static LAST_SELECTION: AtomicU64 = AtomicU64::new(0);     // shape_id or u64::MAX
pub static LAST_SELECTION_ACTION: AtomicU8 = AtomicU8::new(0); // 0=none, 1=toggled_on, 2=toggled_off, 3=cleared
```

`rust_poll_selection()` reads both atomics (swap to 0) and returns `(id, action)` to C. The C bridge maps this to Janet values:

| Action | Janet return value |
|--------|-------------------|
| 0 (none) | `nil` |
| 1 (toggled_on) | `shape_id` (as number, backward compat) |
| 2 (toggled_off) | `[:deselected shape_id]` |
| 3 (cleared) | `:deselected` keyword (fixes existing bug where `nil` was returned) |

**Alternatives considered:**
- `Mutex<Vec<SelectionEvent>>`: Richer but introduces a lock in the render-dispatch path.
- Crossbeam channel: Overkill for <1 event per 100ms.

### D4: Modifier dispatch in handle_click

```python
def handle_click(state, picked_id):
    if picked_id is None:
        # Click on empty space
        if not state.modifiers.control and not state.modifiers.shift:
            clear_selection()
        # else: no-op
    else:
        if state.modifiers.control:
            toggle(picked_id)         # remove if present, add if absent
        elif state.modifiers.shift:
            add(picked_id)            # add unconditionally
        else:
            replace_with(picked_id)   # clear then add
```

`state.modifiers` is already populated by `WindowEvent::ModifiersChanged` (`app.rs:1035-1037`).

### D5: Uniform highlight for all selected shapes

The existing `fs_highlight` pipeline is applied to every shape whose `shape_id` is in `selected_ids`. No primary/secondary or multi-color distinction. Edge active/inactive split uses the same `.contains()` check.

### D6: ViewerToRepl enum updated for documentation

Even though the mpsc receiver is currently discarded, update the enum to `SelectionChanged { ids: Vec<ShapeId>` to document the intent. No actual message processing changes.

## Risks / Trade-offs

- **Drag threshold tuning**: Too small → accidental selection. Too large → deliberate clicks missed. Standard 3px threshold should work. Can be adjusted if reported.
- **janet polling interval (0.1s)**: Multiple rapid selections within 100ms could be coalesced. In practice, a human can't click >10 times/second, and each click now fires on release (longer cycle), so this is acceptable.
- **AtomicU8 for action**: If a future change adds more than 4 action types, this needs expansion. Fine for now.
