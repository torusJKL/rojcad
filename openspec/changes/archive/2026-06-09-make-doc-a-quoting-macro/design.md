## Context

The `doc` function in `boot.janet` looks up documentation for CAD functions. It currently evaluates its argument normally, so `(doc box)` resolves `box` to its value (a binding table), not the symbol `box`. Users must write `(doc 'box)`, which is atypical for Janet — upstream's `doc` is a macro that quotes its argument automatically.

Internally, `doc` is also called programmatically in the `gen-markdown` and `gen-html` documentation generators, where it receives a symbol from a loop variable. A macro (which quotes its arg) would break these internal callers.

## Goals / Non-Goals

**Goals:**
- Make `(doc box)` work at the REPL (auto-quote the symbol)
- Keep `(doc 'box)` working (backward compat)
- Keep internal callers working — programmatic calls with variables
- Zero breakage of user-facing API

**Non-Goals:**
- Changing the return type or output format of `doc`
- Adding no-arg or string-search modes (upstream `doc` has these; not adding here)
- Any C/Rust changes

## Decisions

**Decision: Use upstream's pattern — macro + helper function**

Upstream Janet already solves this exact problem the same way: `doc` is a macro (quotes for users), `doc*` is a function (unquoted, for programmatic use). We mirror this:

1. Rename `defn doc` → `defn get-doc` (internal function, takes a symbol)
2. Add `defmacro doc [sym]` → expands to `(get-doc ',sym)` (quotes for the user)
3. Update internal callers (`gen-markdown`, `gen-html`) to use `get-doc`

**Alternatives considered:**

- **Function + env scan**: Check if arg is a symbol or table; if table, try to reverse-lookup in `core-env` to find its key. Rejected: O(n) per call, ambiguous with duplicate C function pointers.
- **Function + type check**: Accept either symbol (direct lookup) or binding table (extract `:doc` field). Would handle `(doc box)` if `box` evaluated to the binding table, but `box` evaluates to the raw C function, not the binding table.
- **Current behavior (do nothing)**: Users must remember `(doc 'box)`.

**Unquote-handling choice**: The macro will strictly quote its argument. `(doc 'box)` would double-quote to `''box` and fail. This matches upstream Janet's convention — a quoting macro expects unquoted symbols. We choose consistency over defensive handling.

## Risks / Trade-offs

- **Risk**: `(doc 'box)` stops working. **Mitigation**: This matches upstream convention. The REPL help text and examples should show `(doc box)` form.
- **Risk**: Someone calls `(doc)` with no arg — this was already broken (our `defn` required an arg) and remains broken with the macro.
- **Risk**: Internal callers might be added in the future that use `doc` instead of `get-doc`. **Mitigation**: The naming mirrors upstream's `doc`/`doc*` pattern — `doc` is for interactive use, the function form is for code.
