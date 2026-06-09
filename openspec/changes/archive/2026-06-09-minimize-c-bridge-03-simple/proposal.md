## Why

Functions like `quit-requested`, selection polling, edge styling, and view control are thin wrappers around Rust FFI. Moving them to Janet eliminates ~97 lines of C. Most are 5-10 line JANET_FN functions doing get/set on atomics or simple struct construction.

## What Changes

Move 12 functions from C JANET_FN to boot.janet wrappers:
`quit-requested`, `on-select`, `poll-selection`, `selected-shapes`, `list-shapes`, `edge-thickness`, `edge-color-inactive`, `edge-color-active`, `view-fit`, `view-fit-all`, `view-angle`

Strip the corresponding C JANET_FN implementations. No behavior change.

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Additional thin C primitives for selection, edge styling, and view control FFI.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~97 lines removed
- `boot.janet`: ~120 lines added
- `src/main.rs`: No change
