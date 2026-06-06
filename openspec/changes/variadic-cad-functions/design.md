## Context

CAD functions are registered in Janet's root environment by `cad_register_functions(bridge.c)` as `JanetReg` entries. Each function is stored in the env as a table `{:value <C-cfunction> :doc "..." :source "rojcad" :category "..."}`. Three discovery functions (`doc`, `cad-fns`, `group`) read these tables. Two filter helpers (`all-fns`, `apropos`) check `(type :value) = :cfunction`.

The goal is to wrap these C functions with Janet-level variadic versions while preserving the metadata tables so discovery tools continue to work.

## Goals / Non-Goals

**Goals:**
- `hide`, `show`, `purge`, `registry-remove` accept N shape args, apply to each, return nil
- `shape-type`, `visible?`, `wire?`, `face?`, `solid?` accept N shape args, return a tuple of results
- `cut`, `common`, `fuse` accept 1 tool + N operand shapes, chain sequential operations
- Keywords (`:eager`, `:hide`) on boolean ops apply to the final chained operation only
- All metadata (doc, source, category) preserved for `doc`, `cad-fns`, `group`
- `all-fns` and `apropos` discover wrapped functions

**Non-Goals:**
- No changes to transform functions (`translate`, `rotate`, `scale`, `mirror`) — their positional numeric args make variadic ambiguous
- No changes to I/O functions (`write-step`, `write-stl`, `read-step`)
- No changes to wire or sketch operations
- No changes to C or Rust code

## Decisions

### D1 — Implementation in Janet (boot.janet) over C (bridge.c)

| Criterion | Janet | C |
|-----------|-------|---|
| Code complexity | ~50 lines | ~80-100 lines |
| GC safety for boolean chaining | Natural (Janet GC) | Manual root management on C stack |
| Metadata preservation | In-place table mutation | Unchanged (C tables stay intact) |
| Discovery tools | Need minor filter updates | Unchanged |

**Decision**: Janet wrappers. The boolean chaining GC concern in C is real — intermediates are allocated via `janet_abstract` but not returned, and the C stack may not be scanned for raw `void*` pointers during GC. In Janet, chaining is just `each` + `(_cut result b)` — GC handles everything.

### D2 — Table mutation (swap :value, keep table) over def shadowing

Two approaches:
- **def shadowing**: `(def _hide hide) (defn hide [& s] ...)` — replaces the table with a bare function, losing metadata
- **Table mutation**: grab the table, put new function as `:value`, keep table intact

```janet
;; Table mutation approach
(def hide-table (get core-env 'hide))
(def _hide       (hide-table :value))
(put hide-table :value (fn [& shapes] (each s shapes (_hide s))))
```

**Decision**: Table mutation. `doc`, `cad-fns`, `group` read `:doc`, `:source`, `:category` from the table — they see zero change. Only `all-fns` and `apropos` need updating because they check `(type :value) = :cfunction` which no longer matches.

### D3 — Queries return tuple for all arities (including single)

Alternatives considered:
- Return scalar for single shape, tuple for multiple — inconsistent
- Return tuple for all shapes, including one — consistent, but breaking

**Decision**: Return tuple for all. `(shape-type (box 10))` → `@[:solid]`. Breaking change, but consistent and composable.

### D4 — Boolean chaining: keywords on final call only

For `(cut tool a b :eager :hide)`:
- `_cut result a` — binary, no keywords
- `_cut result b :eager :hide` — final call gets both keywords
- Intermediate results are never eagerly tessellated (not needed) and never hidden (only final visibility matters)

If no operand shapes provided (`(cut tool)`), return the tool unchanged.

## Risks / Trade-offs

- **[Breaking] Query return type**: Code that destructures `(def t (shape-type s))` and expects a keyword will break. Code that uses it inline (e.g., `(if (= :solid (shape-type s)) ...)`) will also break. **Mitigation**: Documented in proposal as breaking.
- **[Complexity] Keyword routing for booleans**: The wrapper must separate keyword args from shape args in the rest args, then append keywords to the final binary call. Straightforward in Janet but adds ~10 lines of plumbing per boolean function.
- **[Ripple] Discovery tools**: `all-fns` and `apropos` filter by `(type :value) = :cfunction` — wrapped functions are now `:function`, not `:cfunction`. The filter needs an additional `(or (= (get v :source) "rojcad") ...)` check.
- **[Test gap]**: Existing tests in `cad.rs` test the Rust API directly, not the Janet wrappers. No automated test catches wrapper bugs. Manual REPL testing needed.
