## Why

Users connected to the rojcad TCP REPL via `nc` currently have no way to discover available functions or read their documentation. 30 CAD functions are registered but invisible unless you read the C source. This makes the REPL unfriendly to newcomers and slows down experienced users.

## What Changes

- Add `(all-fns)` — list all available cfunctions in the environment
- Add `(apropos pattern)` — search cfunctions by name substring
- Add `(doc symbol)` — print the docstring for a registered function
- Add `(cad-fns)` — list only the rojcad-specific cfunctions
- Add `(group category)` — list cad-fns grouped by category (e.g., primitives, booleans, transforms)
- Tag CAD functions at C registration time with a `:source "rojcad"` metadata entry so Janet-level helpers can distinguish them from core library functions
- Tag CAD functions with a `:category` metadata entry (e.g., `"primitives"`, `"booleans"`) for programmatic grouping
- Provide a `cad-groups` lookup table in boot.janet with a fallback so any untagged function shows up under `"other"`

## Capabilities

### New Capabilities

- `repl-discoverability`: Functions for listing, searching, and inspecting available CAD functions from the TCP REPL, including docstring access and category-based grouping

### Modified Capabilities

<!-- No existing capability specs are modified -->

## Impact

- `bridge/bridge.c`: Add ~5-line metadata-tagging pass after `janet_cfuns` to annotate each CAD binding's env entry with `:source` and `:category`
- `boot.janet`: Add ~60 lines of Janet code for `all-fns`, `apropos`, `doc`, `cad-fns`, `group`, and the grouping table
- No external dependencies, no protocol changes, no client changes
