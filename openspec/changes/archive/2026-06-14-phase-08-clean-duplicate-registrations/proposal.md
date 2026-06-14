## Why

The `cfuns[]` array in `bridge/bridge.c` registers each CAD function twice — once with a `_` prefix (the "thin primitive") and once without (the "user-facing" name). The non-underscore registration is immediately overwritten by `boot.janet` which replaces it with a wrapper function. This wastes ~18 C entries in the registration array and ~35 lines of C metadata that are never used. Additionally, 7 thin primitive names (`_bx`, `_cy`, `_tr`, etc.) use cryptic abbreviations that should be normalized for consistency.

This is phase 8 of the minimize-c-bridge series, following phases 01–07 which moved C function implementations to Janet wrappers.

## What Changes

1. **Strip redundant non-underscore C registrations** — Remove 18 `"sphere"`, `"cut"`, etc. entries from the `cfuns[]` array where an underscore-prefixed thin primitive already exists
2. **Normalize 7 cryptic thin primitive names** — Rename `_bx` → `_init-box`, `_cb` → `_init-cube`, `_bfc` → `_init-box-from-corners`, `_cy` → `_init-cylinder`, `_cyfp` → `_init-cylinder-from-points`, `_cydir` → `_init-cylinder-point-dir`, `_tr` → `_init-torus`
3. **Update boot.janet captures** — Switch `(get core-env 'sphere')` to `(get core-env '_sphere')` for all Pattern-A functions, and update references to renamed primitives
4. **Remove the `cad_fn_categories` metadata table and loop** (~35 lines of C) — this is dead code since boot.janet already sets `:source` and `:category` on its wrappers
5. **Add missing `:source`/`:category` metadata** to ~10 Janet wrappers in boot.janet that currently lack it
6. **Rename non-underscore C function `quit-requested`** — it currently has a `_` variant too; strip the non-underscore C registration and update the Janet wrapper to reference `_quit-requested`

No behavior change. All public API names remain identical. The only visible effect is internal code quality.

## Capabilities

### New Capabilities

None — this is a pure refactoring with no new user-facing behavior.

### Modified Capabilities

None — no spec-level behavior changes.

## Impact

- `bridge/bridge.c`: ~40 lines removed (18 registration entries + metadata table + loop)
- `boot.janet`: ~20 lines changed (capture names + metadata additions)
- `src/main.rs`, `src/bridge.rs`: No change
- All public API names (`sphere`, `box`, `cut`, `translate`, etc.) remain unchanged
- Thin primitive `_` names change for 7 entries (e.g., `_bx` → `_init-box`), but these are internal-only (only referenced from boot.janet)
