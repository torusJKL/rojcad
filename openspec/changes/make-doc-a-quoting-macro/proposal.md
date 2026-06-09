## Why

The `doc` function in `boot.janet` is a plain function that evaluates its argument normally, which means `(doc box)` fails — `box` evaluates to its binding table, not the symbol `box`. Users must write `(doc 'box)` instead, which is inconsistent with Janet convention (upstream's `doc` is a macro that quotes for you) and confusing.

## What Changes

- Rename the existing `doc` function to `get-doc` (internal use only)
- Add a new `doc` macro that quotes its argument and delegates to `get-doc`
- Update all internal call sites in `boot.janet` to use `get-doc`
- This is an internal refactor — no breakage of the user-facing `doc` API

## Capabilities

### New Capabilities

*(none — this is an internal refactor with no new capabilities)*

### Modified Capabilities

*(none — no existing specs are affected)*

## Impact

- **Single file**: `boot.janet` (rojcad's REPL boot script)
- **No C/Fortran/Rust code changes** — pure Janet change
- **No external API changes** — `doc` remains available at the REPL, same signature
- **Internal function renamed**: `get-doc` replaces `doc` for programmatic use
