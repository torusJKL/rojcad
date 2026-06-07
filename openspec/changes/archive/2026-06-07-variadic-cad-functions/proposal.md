## Why

CAD operations like `hide`, `cut`, and `shape-type` currently accept exactly one shape argument, requiring tedious repetition or manual looping in user code. Making them variadic gives users a more natural, concise syntax — `(hide a b c)` instead of `(do (hide a) (hide b) (hide c))`.

## What Changes

- **Side-effect functions** (`hide`, `show`, `purge`, `registry-remove`) become variadic: apply the effect to each shape argument, return nil. Zero args → nil (no-op).
- **Query functions** (`shape-type`, `visible?`, `wire?`, `face?`, `solid?`) become variadic: apply to each shape, return a tuple of results.
  - **BREAKING**: Single-shape queries now return a tuple: `(shape-type (box 10))` → `@[:solid]` instead of `:solid`.
- **Boolean functions** (`cut`, `common`, `fuse`) become variadic: `(cut tool b1 b2)` = cut b1 from tool, then cut b2 from the result. `:eager` and `:hide` keywords apply only to the final chained operation.
- `(cut tool)`, `(common a)`, `(fuse a)` with a single shape return the shape unchanged (no-op).
- Implementation is in Janet (`boot.janet`) — wrappers that shadow the C functions and mutate the env table's `:value` field to preserve metadata.
- Both `all-fns` and `apropos` in `boot.janet` are updated to recognize the wrapped functions (they currently filter for `:cfunction` type only).

## Capabilities

### New Capabilities

- `variadic-side-effects`: `hide`, `show`, `purge`, `registry-remove` accept multiple shapes
- `variadic-queries`: `shape-type`, `visible?`, `wire?`, `face?`, `solid?` accept multiple shapes, return tuple
- `variadic-booleans`: `cut`, `common`, `fuse` chain multiple shapes
- `doc-meta-preservation`: wrappers preserve docstrings, source tags, and categories so discovery tools continue to work

### Modified Capabilities

- *(none — all capabilities are new)*

## Impact

- **File**: `boot.janet` — ~40 lines added for wrappers, ~10 lines changed in `all-fns` and `apropos`
- **No changes** to `bridge/bridge.c`, `cad.rs`, `main.rs`, `types.rs`, or any Rust code
- **Documentation**: function usage strings in doc generation need to reflect variadic signatures
