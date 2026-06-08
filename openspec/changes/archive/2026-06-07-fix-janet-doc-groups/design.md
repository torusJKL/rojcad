## Context

The `boot.janet` script in rojcad generates Markdown and HTML API documentation from runtime Janet metadata. The `gen-markdown` and `gen-html` functions iterate a dictionary called `cad-groups` which maps internal category keys (e.g., `"primitives"`) to display names (e.g., `"Primitives"`). Only 9 of 14 category keys defined in `bridge/bridge.c`'s `cad_fn_categories` table are present in `cad-groups`, causing 22 functions to be silently dropped from docs.

Additionally, 6 functions registered via `janet_cfuns` in the `cfuns` array have no entry in `cad_fn_categories`, so they never receive `:source "rojcad"` or `:category` metadata — making them invisible to the `(group)` helper entirely.

## Goals / Non-Goals

**Goals:**
- All 22 functions from the 5 missing groups (sketch, 2d-primitives, text, wire-operations, operations) appear in generated docs under appropriate headings
- The 6 orphaned functions (`quit-requested`, `edge-hidden-toggle`, `edge-hidden-show?`, `edge-hidden`, `projection-toggle`, `projection-perspective`) receive `:source "rojcad"` and `:category` metadata so they appear in docs
- Any future category key added to `cad_fn_categories` in bridge.c but forgotten in `cad-groups` renders under an "Other" section rather than being silently dropped
- Doc generation calls `(group)` once instead of once per category (performance optimization)

**Non-Goals:**
- No changes to docstring format in `bridge/bridge.c` JANET_FN macros
- No restructuring of the `boot.janet` doc generation pipeline (same output format, just more complete)
- No external dependencies

## Decisions

### 1. Add missing display names to cad-groups (boot.janet side)
**Chosen:** Add 5 entries to the existing `cad-groups` dictionary.
**Rationale:** This is the existing mechanism for mapping category keys to display names. Adding entries is consistent with how 9 existing categories work. No new machinery needed.

### 2. Tag orphaned functions via cad_fn_categories (bridge.c side)
**Chosen:** Add 6 entries to the `cad_fn_categories` table before the `{NULL, NULL}` sentinel.
**Rationale:** The metadata loop at lines 2406-2414 iterates `cad_fn_categories` to set `:source` and `:category` on function bindings. Adding entries here is the only way those 6 functions get `:source "rojcad"` set, which is required for `(group)` and `(cad-fns)` to discover them.

### 3. Fallback via computed "Other" section (boot.janet side)
**Chosen:** After iterating known `cad-groups`, compute the set of categories from `(group)` that don't have a mapping in `cad-groups` and render all their functions under a single "## Other" section.
**Alternatives considered:**
- Adding `"other" → "Other"` to `cad-groups` directly — this only covers functions with no category, not genuinely unknown future categories.
- Per-unknown-category sections (e.g., "## Wire Operations (unmapped)") — more informative but inconsistent with the single "Other" convention.
**Rationale:** A single "Other" section is the simplest safety net. It avoids cluttering the TOC with raw category keys and is self-documenting: if "Other" appears, the developer knows they forgot to add an entry to `cad-groups`.

### 4. Single `(group)` call optimization
**Chosen:** Call `(group)` once and reuse the result table for both known categories and the fallback.
**Rationale:** `(group cat-k)` iterates the entire Janet environment each time, filtering for `:source "rojcad"` and checking category. Calling it once for all categories avoids O(N²) behavior as the function count grows.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Adding orphaned functions to `cad_fn_categories` with wrong category could misplace them | Categories chosen match their logical group: `edge-hidden-*` → `edge-styling`, `projection-*` → `view`, `quit-requested` → `view` |
| "Other" section could grow large if many future categories are added without being mapped | The section is a safety net. Adding the proper entry to `cad-groups` is trivial — "Other" signals the omission. |
| Functions with no category (category = `nil`) would still fall through to `"other"` key | The fallback catches `"other"` since it's not in `cad-groups`. Those functions appear under "Other" as well. |
