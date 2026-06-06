## Context

The rojcad TCP REPL provides a raw Janet environment with 30 CAD functions registered via `janet_cfuns` in `bridge/bridge.c`. Functions are stored in the environment table as `{:value <cfunction> :doc <string>}` binding tables. Core library functions (math, string, io, etc.) share the same environment, stored in the same format.

Currently there is no way to:
- List available functions
- Search for functions by name
- Read docstrings from the REPL
- Distinguish CAD functions from core library functions

The `janet_core_env` table is accessible from Janet code via `(fiber/getenv (fiber/current))` and is captured as `core-env` in `boot.janet`.

## Goals / Non-Goals

**Goals:**
- Add `(all-fns)`, `(apropos)`, `(doc)`, `(cad-fns)`, and `(group)` for REPL discoverability
- Tag CAD functions with `:source "rojcad"` and `:category` metadata at C registration time
- Provide a grouping table in boot.janet with fallback for ungrouped functions

**Non-Goals:**
- Tab completion — deferred
- Protocol changes — all changes are in-function, not protocol-level
- Client-side changes — `nc` continues to work unchanged

## Decisions

### 1. Metadata tagging approach: C-level post-processing

**Decision**: After `janet_cfuns` in `bridge/bridge.c`, iterate the registered functions and add `:source` and `:category` keywords to each binding's environment table.

**Rationale**: Pure Janet approaches (heuristics, hardcoded name lists) are fragile or require manual maintenance. C-level tagging is a one-time cost that ensures CAD functions are always properly identified regardless of changes to the function list. The C code is adjacent to the registration array, so it's natural to update when adding new functions.

**Alternative considered**: Hardcoding all 30 names in `boot.janet` — simpler but requires manual sync.

### 2. Grouping strategy: boot.janet table with C-level category tag

**Decision**: Categories are set in C at registration time via a `:category` keyword on each binding. A `cad-groups` table in `boot.janet` maps categories to human-readable group names. The `(group)` helper aggregates by category, and any function without a recognized category falls into `"other"`.

**Rationale**: The C tag is authoritative (no sync issues). The Janet-level table is easy to read and modify. The `"other"` fallback ensures new functions are never invisible even if their category tag is missing or unknown.

### 3. Environment iteration strategy

**Decision**: Use `(next core-env nil)` / `(next core-env prev-key)` for iteration in all helpers.

**Rationale**: `(next)` is a primitive and doesn't allocate intermediate collections. `(pairs)` creates a fiber that may be slower. For a REPL helper invoked once per user request, performance is not critical, but using primitives is idiomatic Janet.

## Risks / Trade-offs

- **[Maintenance] Adding a new CAD function requires updating both the `cfuns[]` array and the category table in `bridge/bridge.c`** — Mitigation: Both changes are in the same file, on adjacent lines. The `"other"` fallback means a forgotten category tag won't break anything, just show the function as uncategorized.
- **[Compatibility] New functions use the `core-env` captured at boot time** — Mitigation: `core-env` is captured after all registration, so all CAD functions are present. User `def`'d symbols at the REPL also appear since they use the same table via `fiber/setenv`.
