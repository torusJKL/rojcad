## Why

The Janet REPL can currently only create two primitives (box, sphere) with inconsistent calling conventions. OCCT provides a rich set of primitives — cylinder, cone, torus — that users have no way to create. This change expands coverage to all OCCT 3D primitives with a uniform, concise keyword-driven calling convention.

## What Changes

- Rename `make-box` to `box` and `make-sphere` to `sphere`
- Add `cylinder`, `cone`, `torus` functions
- Every function supports positional args for common cases and keyword args for explicit parameter naming
- Keywords use short abbreviations: `:w`, `:d`, `:h`, `:c`, `:r`, `:a`, `:dir`, etc.
- `:center` uses array syntax `[x y z]` instead of tuple `'(x y z)`
- Expose all OCCT constructor variants per primitive (box_from_corners, cylinder_from_points, partial torus, etc.)
- All primitives also available as cube, cube-centered via single-arg box and keywords
- **BREAKING**: `make-box` and `make-sphere` are removed; callers must use `box` and `sphere`

## Capabilities

### New Capabilities
- `cad-primitives`: All OCCT 3D primitives (box, sphere, cylinder, cone, torus) with full constructor coverage, uniform keyword API, short parameter abbreviations

### Modified Capabilities
- `cad-primitives`: Renamed functions, changed calling conventions, added :center array syntax

## Impact

- `src/cad.rs`: Add `make_cylinder`, `make_cone`, `make_torus`; update `make_box`/`make_sphere` to support keyword args
- `src/main.rs`: Add `rust_init_cylinder`, `rust_init_cone`, `rust_init_torus` FFI bridge functions
- `bridge/bridge.c`: Replace `make-box`/`make-sphere` with `box`/`sphere`; add `cylinder`, `cone`, `torus` JANET_FN wrappers; update parse_center_keyword callers
- `src/bridge.rs`: Add extern declarations for new `rust_init_*` functions
- Tests in `src/cad.rs`: Existing tests update to new names; add tests for cylinder, cone, torus, all constructor variants, all keyword calling conventions
