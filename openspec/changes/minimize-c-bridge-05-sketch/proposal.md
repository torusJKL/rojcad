## Why

Sketch operations and wire operations are thin wrappers: unwrap sketch, call rust_* FFI, wrap result. Moving to Janet eliminates ~75 lines of C. Sketch operations compose naturally with Janet's threading macro (`->`).

## What Changes

Move 9 sketch functions (`sketch`, `move-to`, `line-to`, `line-dx`, `line-dy`, `line-dx-dy`, `arc-to`, `close-sketch`, `build-wire`) from C JANET_FN to boot.janet.
Move 4 wire operations (`wire-to-face`, `wire-fillet`, `wire-chamfer`, `wire-offset`) from C JANET_FN to boot.janet.
Strip the corresponding C implementations. No behavior change.

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Thin C primitives for sketch operations.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~75 lines removed
- `boot.janet`: ~60 lines added
- `src/main.rs`: No change
