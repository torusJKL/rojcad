## Why

The medium-complexity functions (sphere, cone, 2D primitives, booleans, transforms, text) have non-trivial keyword argument parsing in C. Moving them to Janet eliminates ~300 lines of C while making the argument parsing cleaner and more maintainable. These functions accept both positional and keyword arguments (mixed — they have `:eager` and `:hide` keywords), requiring manual variadic parsing in Janet (~15 lines each).

## What Changes

Move 18 functions from C JANET_FN to boot.janet:
`sphere`, `cone`, `extrude`, `revolve`, `extrude-polygon`, `rect`, `circle`, `polygon`, `text`, `text3d`, `list-fonts`, `cut`, `common`, `fuse`, `translate`, `rotate`, `scale`, `mirror`

Strip the corresponding C JANET_FN implementations. No behavior change.

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Thin C primitives for medium-complexity CAD operations.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~300 lines removed
- `boot.janet`: ~240 lines added
- `src/main.rs`: No change
