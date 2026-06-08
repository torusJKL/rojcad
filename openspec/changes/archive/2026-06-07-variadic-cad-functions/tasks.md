## 1. Side-effect wrappers (hide, show, purge, registry-remove)

- [x] 1.1 Add table-mutation wrapper for `hide` — apply to each shape arg, return nil
- [x] 1.2 Add table-mutation wrapper for `show` — apply to each shape arg, return nil
- [x] 1.3 Add table-mutation wrapper for `purge` — apply to each shape arg, return nil
- [x] 1.4 Add table-mutation wrapper for `registry-remove` — apply to each shape arg, return nil

## 2. Query wrappers (shape-type, visible?, wire?, face?, solid?)

- [x] 2.1 Add table-mutation wrapper for `shape-type` — return tuple of results
- [x] 2.2 Add table-mutation wrapper for `visible?` — return tuple of results
- [x] 2.3 Add table-mutation wrapper for `wire?` — return tuple of results
- [x] 2.4 Add table-mutation wrapper for `face?` — return tuple of results
- [x] 2.5 Add table-mutation wrapper for `solid?` — return tuple of results

## 3. Boolean wrappers with keyword chaining (cut, common, fuse)

- [x] 3.1 Add table-mutation wrapper for `cut` — chain operands, route keywords to final call, single-operand no-op
- [x] 3.2 Add table-mutation wrapper for `common` — chain operands, route keywords to final call, single-operand no-op
- [x] 3.3 Add table-mutation wrapper for `fuse` — chain operands, route keywords to final call, single-operand no-op

## 4. Discovery tool updates

- [x] 4.1 Update `all-fns` to discover wrapped functions (check `:source` property as well as `:cfunction` type)
- [x] 4.2 Update `apropos` to discover wrapped functions (same check as `all-fns`)

## 5. Verification

- [x] 5.1 Connect via REPL and verify `(hide a b c)` hides all three shapes
- [x] 5.2 Connect via REPL and verify `(shape-type a b)` returns `@[:solid :solid]`
- [x] 5.3 Connect via REPL and verify `(cut tool a b)` produces a valid chained result
- [x] 5.4 Connect via REPL and verify `(doc 'hide)` still returns documentation
- [x] 5.5 Connect via REPL and verify `(cad-fns)` includes wrapped functions
- [x] 5.6 Connect via REPL and verify `(all-fns)` includes wrapped functions
- [x] 5.7 Connect via REPL and verify `(apropos "hide")` finds the wrapped function
- [x] 5.8 Connect via REPL and verify zero-arg calls `(hide)` return nil without error
