## Context

The parametric model system (`defmodel`, `build`, `graph`, `highlight`, `highlight-clear`) was implemented in `boot/model.janet` but the functions were never tagged with `setmeta` for the doc generation pipeline. The doc pipeline scans `core-env` for entries with `:source = "rojcad"` and a known `:category`, then extracts `:doc` strings. Without these metadata fields, the functions are invisible to `dump-docs`.

The README shows imperative CAD usage but has no mention of parametric models — the feature that distinguishes rojcad from a raw OCCT wrapper.

## Goals / Non-Goals

**Goals:**
- All parametric model functions appear in `doc/janet-api.md` and `doc/janet-api.html` with usage, examples, and return types
- README has a "Parametric Models" section showing the end-to-end workflow
- Existing doc generation infrastructure is reused (no new doc tools)

**Non-Goals:**
- Changing the parametric model implementation or API surface
- Adding doc generation for macros (`defmodel` is a macro — handle via the Janet binding)
- Interactive model tree UI in docs (future work)
- Tutorials, guides, or example galleries beyond the README section

## Decisions

### Decision 1: Add `setmeta` calls inline in `model.janet`

**Chosen**: Add `setmeta` calls after each function/macro definition in `boot/model.janet`, plus a new `"parametric-models"` category entry in `cad-groups` in `boot.janet`.

**Rationale**:
- Follows the exact same pattern as `boot.janet` — every wrapper function there has a `defmeta` or `setmeta` call
- The `setmeta` call is simple: `(setmeta 'fn-name "category" "docstring")`
- No structural changes needed — just metadata annotation
- The doc generation pipeline (`dump-docs` → `group()` → `get-doc()`) works unchanged

**Alternatives considered**:
- Splitting model functions into `boot.janet` — would mix concerns (REPL server vs model layer)
- Auto-tagging via a convention in `dump-docs` — more complex, less explicit

### Decision 2: README section — workflow narrative, not comprehensive reference

**Chosen**: Add a "Parametric Models" section showing a single concrete workflow: define a bracket model, build it with parameters, introspect with `graph`, highlight a part, rebuild with new params. Link to generated API docs for full reference.

**Rationale**: README is the first thing users see — it should sell the feature, not document it exhaustively. A narrative workflow shows the power of the parametric system in ~20 lines of code.

### Decision 3: Macros get `setmeta` like functions

**Chosen**: `defmodel` is a macro (defined via `defmacro`), but it still creates a binding in `core-env` with a `:value` field. `setmeta` sets metadata on that binding table just like any function. The doc pipeline doesn't distinguish between macros and functions — it only checks `:source` and `:category`.

**Rationale**: Works without special-casing. The generated docs will show `defmodel` alongside regular functions with its usage signature.

## Risks / Trade-offs

- **`defmodel` is a macro, not a function** — its `:doc` string describes the macro syntax, but `dump-docs` labels it the same as functions. Users familiar with Janet macros might expect a "macro" tag, but this is consistent with how Janet's own core docs work.
- **Docstring format** — the docstrings in `model.janet` need to follow the same conventions as `boot.janet` (usage as first paragraph, then body, "Examples:" section, "Returns" line). `dump-docs`'s `split-docstring` function parses this structure.
