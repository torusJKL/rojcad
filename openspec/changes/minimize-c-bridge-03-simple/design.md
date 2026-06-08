## Context

These functions range from trivial (quit-requested: read atomic) to more complex (poll-selection: construct tuples from Rust atomics, invoke callback). The edge styling and view functions are mostly get/set wrappers. `selected-shapes` and `list-shapes` iterate IDs and construct Janet tuples of abstract values.

## Key Decisions

### `poll-selection` and `on-select`

The callback storage (`on_select_callback`) is currently a global static in bridge.c. This needs to move to Rust or be managed differently. Options:
1. **Move the callback to a Rust global** (e.g., `OnceLock<Mutex<Option<...>>>`)
2. **Keep the callback in Janet** — Janet already stores the callback; the C function just invokes it

Option 2 is simpler: the Janet wrapper for `on-select` stores the function in a Janet `def`, and the `poll-selection` wrapper calls it if set. The C primitive just returns the atomic value.

### `selected-shapes` and `list-shapes`

These need to construct tuples of Janet abstract values from Rust ID arrays. The C code currently iterates IDs, looks up pointers, and wraps them. This can go to Janet if we expose two thin primitives: one to get the IDs (returns array of ints) and one to resolve an ID to a shape. Or keep as a single C primitive that returns a tuple.

Simpler approach: expose `rust-get-selected-ids` and `rust-get-registered-ids` as C primitives returning Janet arrays of ints, then construct the shapes in Janet.

## Functions

| Function | Pattern | Notes |
|----------|---------|-------|
| quit-requested | 0-arg → bool | Trivial atomic read |
| on-select | 1-arg fn/nil | Stores callback in Janet world |
| poll-selection | 0-arg → value | Returns nil/:deselected/[:deselected id]/id, invokes stored callback |
| selected-shapes | 0-arg → tuple | IDs → shape lookup → wrap |
| list-shapes | keywords → tuple | Filter {visible,hidden} + ID lookup |
| edge-thickness | 0-1 arg → number | Get/set |
| edge-color-inactive | 0-3 args → nil | Set RGB |
| edge-color-active | 0-3 args → nil | Set RGB |
| view-fit | shapes + :reset → nil | Variadic, alloc array, call FFI |
| view-fit-all | keywords → nil | :hidden :reset |
| view-angle | 2-3 args → nil | yaw pitch [distance] |
