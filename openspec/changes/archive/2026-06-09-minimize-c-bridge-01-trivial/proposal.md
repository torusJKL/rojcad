## Why

Strip the simplest C bridge functions — edge visibility toggles, projection mode, overlay visibility, and window state getters/setters. These are 1:1 wrappers calling into Rust atomics with zero argument parsing logic. Moving them to Janet eliminates ~54 lines of C boilerplate with virtually no risk.

## What Changes

- Move 19 getter/setter functions from bridge/bridge.c (C JANET_FN) to boot.janet (Janet wrappers around thin C primitives)
- Strip the corresponding C JANET_FN implementations from bridge.c
- No behavior change — the Janet wrappers produce identical results

Functions moved:
`edge-toggle-inactive`, `edge-toggle-active`, `edge-inactive-show?`, `edge-active-show?`, `edge-hidden-toggle`, `edge-hidden-show?`, `edge-hidden`, `projection-toggle`, `projection-perspective`, `stats-overlay`, `window-help-toggle`, `window-help-show?`, `window-help-show`, `window-size`, `window-size?`, `window-fullscreen`, `window-fullscreen?`, `window-maximized`, `window-maximized?`

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Thin C JANET_FN wrappers callable from Janet — one per rust_* FFI function. These are the primitive building blocks that Janet code composes.

### Modified Capabilities

None — pure refactoring, no behavior change.

## Impact

- `bridge/bridge.c`: ~54 lines removed
- `boot.janet`: ~60 lines added (19 Janet wrappers)
- `src/main.rs`: No change (the rust_* FFI functions remain unchanged)
