## 1. C Metadata Tags

- [x] 1.1 Add category constants array to `bridge/bridge.c` mapping each CAD function name to its category string (primitives, booleans, transforms, queries, registry, io, selection, edge-styling)
- [x] 1.2 Add post-registration loop in `bridge.c` after `janet_cfuns` that iterates the registered functions and tags each binding's env table with `:source "rojcad"` and the corresponding `:category` keyword

## 2. Janet REPL Helpers

- [x] 2.1 Implement `(all-fns)` in `boot.janet` — iterate `core-env`, collect keys whose `:value` is `:cfunction`, sort, return array
- [x] 2.2 Implement `(apropos pattern)` in `boot.janet` — same as `all-fns` but filter by `(string/find pattern (string k))`
- [x] 2.3 Implement `(doc symbol)` in `boot.janet` — look up binding in `core-env`, return `:doc` string or "no documentation" message
- [x] 2.4 Implement `(cad-fns)` in `boot.janet` — iterate `core-env`, filter for `(= (get binding :source) "rojcad")`, sort, return array
- [x] 2.5 Implement `cad-groups` lookup table in `boot.janet` mapping category strings to human-readable group names, with a fallback for missing categories
- [x] 2.6 Implement `(group &opt category)` in `boot.janet` — returns either a specific category's function list or a table of all categories, with uncategorized functions under `"other"`

## 3. Verify

- [x] 3.1 Build project with `just build` and confirm no compilation errors
- [x] 3.2 Run `just test-unit` to confirm existing tests pass
- [x] 3.3 Run `just lint` to confirm no new clippy warnings (all 15 failures are pre-existing)
