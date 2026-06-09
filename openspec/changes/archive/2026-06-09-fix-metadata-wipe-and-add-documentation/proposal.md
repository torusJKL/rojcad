## Why

Two bugs silently wipe metadata (`:source`, `:category`, `:doc`) from 28 CAD functions after boot. `model.janet` replaces core-env table entries with fresh tables, and the `rojcad-groups` iteration uses function values instead of symbols — both destroying the metadata set by `defmeta`. Additionally, ~48 wrapper functions have no documentation at all, and 13 more have a fragile split-doc pattern (doc put 700 lines before the corresponding `defmeta`).

## What Changes

- **Fix `wrap-c-fn` macro**: Mutate `:value` in-place instead of replacing the entire table entry. Pre-existing metadata survives.
- **Fix manual `put` patterns**: Change `(put core-env 'name @{:value ...})` to `(put entry :value ...)` for `box`, `cylinder`, `torus`, `compound`.
- **Fix `model.janet` wrapping**: Same fix — mutate `:value` in-place instead of replacing the table. Metadata set by `defmeta` earlier in boot survives.
- **Remove `rojcad-groups` iteration**: Dead code. Every function now has its own `defmeta` call.
- **Add `defmeta` + docstrings for 48 functions currently missing them**: All wrapper functions get proper documentation describing the user-facing Janet API (keyword args, variadic behavior, etc.) — not the C thin-primitive API.
- **Merge 13 split doc puts**: Move docstrings from standalone `(put ... :doc ...)` into `defmeta` calls, eliminating the ~700-line fragility gap.
- **Fill 12 existing `defmeta` doc gaps**: Add 3rd argument to `defmeta` calls that currently lack a docstring.

No behavioral changes. No API changes. All function signatures, return values, and semantics are preserved.

## Capabilities

### New Capabilities

*(None — this is an internal fix and documentation improvement. No new capabilities.)*

### Modified Capabilities

*(None — no existing spec-level requirements change.)*

## Impact

- **Files**: `boot.janet` (~80 lines added for `defmeta` + docstrings), `boot/model.janet` (~5 lines changed)
- **No C/Fortran/Rust code changes** — pure Janet fix
- **No external API changes** — all function signatures, return values, and semantics are preserved
- **Runtime**: Zero behavior change; metadata now survives boot and is accurate
