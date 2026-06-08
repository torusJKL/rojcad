## Why

Several visibility and query functions (`show`, `hide`, `purge`, `registry-remove`, `visible?`, `wire?`, `face?`, `solid?`, `shape-type`) are already wrapped in boot.janet for variadic behavior. The underlying C JANET_FN implementations are trivial (1-5 lines each). Stripping them removes ~24 lines of C.

## What Changes

- Strip the C JANET_FN implementations for `show`, `hide`, `purge`, `registry-remove`, `visible?`, `wire?`, `face?`, `solid?`, `shape-type`
- boot.janet already has wrappers for `show`, `hide`, `purge`, `registry-remove` — only need to add thin C primitives for the core Rust FFI calls
- Add new Janet wrappers for `visible?`, `wire?`, `face?`, `solid?`, `shape-type`
- No behavior change

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Additional thin C primitives for visibility and type-query FFI calls.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~24 lines removed
- `boot.janet`: ~24 lines added (core primitives + wrappers)
- `src/main.rs`: No change
