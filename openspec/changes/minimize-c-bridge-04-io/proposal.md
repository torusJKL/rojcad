## Why

STEP/STL export and STEP import are simple string-argument functions with error handling. Moving them to Janet eliminates ~20 lines of C.

## What Changes

Move `write-step`, `write-stl`, `read-step` from C JANET_FN to boot.janet wrappers. Strip the corresponding C implementations. No behavior change.

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Thin C primitives for file I/O FFI.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~20 lines removed
- `boot.janet`: ~20 lines added
- `src/main.rs`: No change
