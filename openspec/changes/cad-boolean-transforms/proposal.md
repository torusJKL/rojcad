## Why

The CAD system has boolean `cut` and `common` operations but is missing `fuse` (union). It has no transformation operations at all — users cannot translate, rotate, scale, or mirror shapes. These are fundamental CAD operations needed for any non-trivial modeling workflow.

## What Changes

- **Add `fuse` (union)** boolean operation — the only missing boolean op
- **Add `translate`** — create a translated copy of a shape (pure, no in-place mutation)
- **Add `rotate`** — create a rotated copy about a specified axis
- **Add `scale`** — create a uniformly scaled copy about a point
- **Add `mirror`** — create a mirrored copy about an axis
- **Change `:a` semantic** — `:a` keyword now means **degrees** (was radians). **BREAKING**.
- **Add `:ar` keyword** — for specifying angles in **radians** as an alternative to `:a`
- **Add `:asr`/`:aer` keywords** on `torus` — radian equivalents of `:as`/`:ae`. **BREAKING**: `:as`/`:ae` now mean degrees.
- Fork `opencascade-rs` to add `translated`, `rotated`, `scaled`, `mirrored` methods to `Shape`

## Capabilities

### New Capabilities
- `cad-booleans-fuse`: Union (fuse) operation for two shapes
- `cad-transforms`: Pure transformation operations — translate, rotate, scale, mirror

### Modified Capabilities
- `<existing-name>`: No existing spec files are being modified. The angle keyword changes are implementation details of existing primitives, documented here rather than via spec-level deltas.

## Impact

- **opencascade-rs fork**: Add 4 methods to `Shape` (translated, rotated, scaled, mirrored) using `BRepBuilderAPI_Transform`
- **Cargo.toml**: Point `opencascade` dependency to fork URL
- **src/cad.rs**: Add `fuse`, `translate`, `rotate`, `scale`, `mirror` functions
- **src/main.rs**: Add 6 `rust_init_*` FFI functions
- **src/bridge.rs**: Add 6 extern declarations
- **bridge/bridge.c**: Add 6 `JANET_FN` wrappers + register; add degrees→radians conversion for `:a`/`:as`/`:ae` keywords
- **Tests**: New unit tests for all operations; verify original shape immutability
