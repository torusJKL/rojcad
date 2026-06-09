## Why

`JANET_BOOTSTRAP=1` builds the Janet core environment from C source instead of loading a pre-compiled bytecode image. This means standard Janet macros (`defmacro`, `defn`, `->`, `->>`, `each`, `for`, `loop`, `case`, `match`, etc.) are entirely absent at the runtime REPL. Users coming from standard Janet find this limiting and must work around it with verbose `(def name :macro (fn [...] ...))` patterns.

Embedding upstream Janet's `src/boot/boot.janet` and loading it at startup gives users the full standard macro suite with zero reimplementation effort.

## What Changes

- **Add `upstream.janet`** — the upstream Janet 1.41.2 `src/boot/boot.janet` (164KB), vendored at project root
- **Load `upstream.janet` before `boot.janet`** in `src/main.rs` — two `janet_dostring` calls so that rojcad's own boot code can optionally use the macros
- **Update `AGENTS.md`** — document that standard macros are now available and note the `&form`/`&env` limitation that remains
- No changes to rojcad's `boot.janet` — it continues to work as-is; future refactoring can optionally use the macros

## Capabilities

### New Capabilities
- `janet-core-macros`: Provides the full suite of standard Janet macros (`defmacro`, `defn`, `def-`, `comment`, `->`, `->>`, `-<>`, `-<>>`, `as->`, `with`, `each`, `for`, `loop`, `case`, `match`, `cond`, `let`, `when`, `unless`, `try`, `protect`, `and`, `or`, `seq`, `generate`, `coro`, and many more) at the runtime REPL by loading upstream Janet's `boot.janet` source.

### Modified Capabilities
*(None — no existing spec-level behavior changes)*

## Impact

- **Binary size**: +164KB from embedded `upstream.janet` source string
- **Boot time**: Negligible — parsing 164KB of Janet source is fast, and most of it is just definitions (no heavy computation at load time)
- **Environment**: New names added to `core-env` — verified no conflicts with rojcad CAD function names. The only name overlap is `doc` (upstream defines it as a macro, rojcad overrides it as a function — rojcad's wins since it loads second)
- **`&form`/`&env` limitation**: Remains in place — this is a compiler-level restriction of `JANET_BOOTSTRAP`, not fixable from Janet code
- **Maintenance**: Minimal — if we upgrade the vendored Janet version, we sync `upstream.janet` to match
