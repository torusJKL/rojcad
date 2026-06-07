## Why

When a Janet symbol bound to a shape is redefined, the old shape remains in the viewer's `ShapeRegistry` — still rendered every frame, still consuming GPU memory, but inaccessible because the symbol now points to the new shape. E.g., `(def obj (box 10))` followed by `(def obj (sphere 20))` leaves the box ghost-rendered until Janet GC happens to collect it. This is confusing in an interactive CAD workflow where redefining a symbol should replace — not accumulate — shapes.

## What Changes

- **Auto-purge on symbol redefinition**: When `my-eval` evaluates a top-level `(def name expr)` or `(set name expr)` where `name` was already bound to a `:rojcad/shape`, the old shape is purged (removed from `ShapeRegistry`, unregistered, marked purged) before the new shape is auto-shown.
- **`set` auto-show**: The existing auto-show logic (which currently only applies to `def`) is extended to also cover `set`, so rebinding via `set` also makes the new shape visible.
- **Identity-safe**: When the old and new shape values are the same object (e.g., `(def obj obj)`), no purge occurs — avoids a purge-then-show panic.
- No new keywords, no opt-in, no Rust changes.

## Capabilities

### New Capabilities
- `shape-lifecycle`: Auto-purge of shapes when their binding symbol is redefined via `def` or `set`, with automatic visibility of the new shape

### Modified Capabilities
<!-- No existing specs changed — this is additive behavior on top of shape-visibility -->

## Impact

- `boot.janet`: ~10 lines added to `my-eval` — capture old binding before `compile`/`resume`, purge if it was a shape, extend auto-show condition to `set`.
- No Rust/C changes (pure Janet change, same layer as auto-show-on-def).
- Backward compatible — existing `def`/`set` usage unaffected. Old shape purge is an invisible improvement for the common case.
