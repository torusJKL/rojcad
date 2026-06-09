## Context

The minimize-c-bridge series (phases 01–07) moved CAD function wrappers from C `JANET_FN` implementations into `boot.janet`, keeping thin C primitives. Each phase kept the now-redundant underscore-prefixed registration alongside the non-underscore one. The `cad_fn_categories` metadata table (35 lines) was dead code.

Additionally, 5 thin primitives use cryptic shorthand names (`_bx`, `_cb`, `_bfc`, `_cyfp`, `_cydir`) that don't match the naming convention of the rest.

## Goals / Non-Goals

**Goals:**
- Remove all redundant underscore C registrations (40 entries where `_x` + `x` both exist)
- Normalize 5 cryptic thin primitive names to `_init-*` convention
- Remove the `cad_fn_categories` dead code + its enrichment loop
- Add `:source`/`:category` metadata in boot.janet using category-keyed groups
- All public API names remain unchanged

**Non-Goals:**
- Moving more functions from C to Janet (that was phases 01–07)
- Changing function behavior, signatures, or docstrings
- Modifying the Rust side of the bridge

## Decisions

### 1. Approach choice: keep non-underscore C names, strip underscore ones

**Decision**: Keep non-underscore C registrations as the canonical thin primitives (e.g., `"sphere"`, `"cut"`), strip their underscore duplicates (`"_sphere"`, `"_cut"`). Boot.janet captures from the non-underscore C name, avoiding the name-collision issue where `(def _sphere ...)` overwrites the C-registered `_sphere` symbol.

Two exceptions: `_quit-requested` is stripped in favor of `"quit-requested"`; the internal helpers `_poll-selection-raw`, `_get-selected-ids`, `_get-registered-ids`, `_get-shape` stay as underscore-only.

### 2. Name convention for thin primitives

**Decision**: Rename 5 cryptic underscore-only names to `_init-*` convention. These have no non-underscore C equivalent.

| Current | New | Type |
|---------|-----|------|
| `_bx` | `_init-box` | Box constructor |
| `_cb` | `_init-cube` | Cube case |
| `_bfc` | `_init-box-from-corners` | Opposite corners |
| `_cyfp` | `_init-cylinder-from-points` | From-point/to-point |
| `_cydir` | `_init-cylinder-point-dir` | Point + direction |

The simple cylinder and torus constructors (`_cy`, `_tr`) are stripped — boot.janet captures from the non-underscore `"cylinder"` and `"torus"` entries instead.

### 3. Metadata ownership

**Decision**: Remove all C-level metadata (`cad_fn_categories`). Replace with a compact data-driven approach in boot.janet using a category-to-symbols table iterated with `while`/`next`:

```janet
(def rojcad-groups
  {"primitives" [sphere cone cylinder torus]
   "booleans" [cut common fuse]
   ...})

(var gk (next rojcad-groups nil))
(while gk
  (def cat gk)
  (def syms (get rojcad-groups gk))
  (var i 0)
  (def n (length syms))
  (while (< i n)
    (def t (get core-env (syms i)))
    (when (= :table (type t))
      (put t :source "rojcad")
      (put t :category cat))
    (set i (+ i 1)))
  (set gk (next rojcad-groups gk)))
```

This replaces ~126 repetitive `(put ...)` lines with ~36 lines of table + loop.

### 4. `quit-requested` handling

**Decision**: Strip `"_quit-requested"` C registration; keep `"quit-requested"`. Boot.janet captures from `(get core-env 'quit-requested)` as before.

## Risks / Trade-offs

- **Risk**: A boot.janet capture references a stripped underscore C name. **Mitigation**: boot.janet captures all use the original non-underscore names.
- **Risk**: `cad_fn_categories` removal could break `cad-fns` or `group`. **Mitigation**: Metadata is now set in boot.janet via the groups table, verified via REPL.
- **Risk**: Renaming `_bx` → `_init-box` breaks external Janet code. **Mitigation**: These are documented as internal thin primitives. CHANGELOG entry provided.
