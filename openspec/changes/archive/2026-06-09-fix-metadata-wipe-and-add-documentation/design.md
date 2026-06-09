## Context

The boot sequence loads `boot.janet` and `model.janet` as a single chunk via `janet_dostring`. Within `boot.janet`:

1. C functions are registered by `janet_cfuns` → core-env entries are `{:value <cfn> :doc "C doc"}`
2. `wrap-c-fn` replaces entries with `@{:value (fn ...)}` — loses `:doc` from C
3. `defmeta` adds `:source`, `:category`, `:doc` back
4. `rojcad-groups` iteration is supposed to set `:source`/`:category` as a safety net
5. `model.janet` wraps entries for shape tracking — replaces tables again, wiping metadata

**Root cause 1:** `wrap-c-fn` and the manual `put core-env` patterns use `(put core-env 'name @{:value ...})` which creates a **new** table, destroying any existing keys (`:doc`, `:source`, `:category`).

**Root cause 2:** The `rojcad-groups` iteration uses `(each sym syms)` where `sym` resolves to a function/table value (not a symbol), so `(get core-env sym)` returns nil. The loop is a silent no-op.

**Root cause 3:** `model.janet` uses the same replace-table pattern, wiping metadata for the 28 functions in `cad-shape-fns` — undoing any `defmeta` calls that ran earlier.

## Goals / Non-Goals

**Goals:**
- Every wrapper function retains its `:source`, `:category`, `:doc` through all layers of boot
- Every wrapper function has a `defmeta` call with a correct docstring describing the user-facing API
- Remove the broken `rojcad-groups` iteration
- Merge the 13 split doc-put patterns into their `defmeta` calls
- Fill 12 existing `defmeta` calls that lack docstrings

**Non-Goals:**
- No behavioral changes to any function
- No changes to the C bridge or Rust source
- No new specs (no capability changes)
- No changes to documentation generators or REPL help output format

## Decisions

### 1. Mutate `:value` in-place instead of replacing the table

This is the central fix. Three patterns to change:

**`wrap-c-fn` macro** (lines 13-15 of `boot.janet`):
```
;; Before
(put core-env ',name @{:value (fn ,arglist ,;body)})
;; After
(put (get core-env ',name) :value (fn ,arglist ,;body))
```

**Manual `put` for box/cylinder/torus/compound**:
```
;; Before
(put core-env 'box @{:value (fn [& args] ...)})
;; After
(def box-entry (get core-env 'box))
(put box-entry :value (fn [& args] ...))
```

**`model.janet` wrapping** (lines 38-47):
```
;; Before
(put core-env fn-name @{:value (fn [& args] ...)})
;; After
(put (get core-env fn-name) :value (fn [& args] ...))
```

`color` is a special case — it's a new function name (not a C function), so it still needs the create-table form. Its `defmeta` runs immediately after creation, so metadata survives.

**Rationale**: Minimal change. Every existing `defmeta` call, explicit `(put ... :doc ...)`, and any future metadata additions automatically survive all wrapping layers. No need to copy-and-restore individual keys.

### 2. Remove `rojcad-groups` iteration entirely

Since every function now gets its own `defmeta` call, the group iteration provides zero value. Remove both the data table and the loop.

**Rationale**: Eliminates dead code. The iteration was the only consumer of `rojcad-groups`. Quoting the symbol arrays to fix the iteration would add maintenance burden (keep defmeta categories in sync with groups) for no benefit.

### 3. Every `defmeta` call includes a hand-written docstring

No fallback to C docstrings. Each doc describes the actual Janet wrapper API (keyword args, variadic behavior, named parameters — not the thin-primitive positional API).

**Rationale**: The C docstrings describe `(_sphere radius cx cy cz angle eager hide)` — a different API from `(sphere :r 5 :c [1 2 3] :eager)`. Using them would mislead users. Hand-written docs are the only correct option.

### 4. Add `defmeta` + doc right after each function's wrapper definition

For Tier 1-3 functions, insert `(defmeta name "category" "docstring")` immediately after the line that completes the wrapper definition (the closing `)` of `wrap-c-fn` or the `}` closing the manual table).

**Rationale**: Co-locates definition and documentation. Eliminates the 13 split-doc patterns where the doc put is 700 lines before the metadata registration.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `color` is a new name with no prior C entry — `(get core-env 'color)` returns nil before its `put` | `color` uses the create-table form `(put core-env 'color @{:value ...})` — this is correct for new entries. `defmeta` (which requires an existing table) runs right after. |
| `set-color` is never wrapped — its original C entry is used by `color`'s wrapper | Not affected. `set-color` doesn't appear in `cad-shape-fns` and has no `wrap-c-fn` call. |
| Model wrapping loops over `cad-shape-fns` — any new function added to this list without a prior `defmeta` would have no metadata until its own `defmeta` is added | This is correct behavior: the fix ensures existing metadata survives; missing metadata is a separate concern. |
| Docstring wording might be inconsistent since ~60 docs are being written at once | Acceptable. Docs can be refined in follow-up changes. The priority is having *a correct* doc for every function, not perfection. |
