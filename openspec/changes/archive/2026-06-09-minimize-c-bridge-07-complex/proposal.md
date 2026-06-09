## Why

The three most complex functions (`box`, `cylinder`, `torus`) have the most argument parsing code in C — each handles multiple construction modes (positional, keyword, box/cube variants, fp/tp/dir for cylinder, angle/axis variants for torus). Moving them to Janet eliminates ~180 lines of C. These require the most careful manual variadic parsing (~20 lines each) and the most thorough testing.

## What Changes

Move `box`, `cylinder`, `torus` from C JANET_FN to boot.janet wrappers. Strip the corresponding C implementations. No behavior change.

## Capabilities

### New Capabilities

- `janet-bridge-primitives`: Thin C primitives for the final three CAD operations.

### Modified Capabilities

None.

## Impact

- `bridge/bridge.c`: ~180 lines removed
- `boot.janet`: ~100 lines added
- `src/main.rs`: No change
