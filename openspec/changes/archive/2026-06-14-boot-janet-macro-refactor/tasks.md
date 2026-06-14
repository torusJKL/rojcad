## 1. Helper macros (foundation)

- [x] 1.1 Add `defmeta` macro — single form for `:source` + `:category` + optional `:doc` metadata
- [x] 1.2 Add `wrap-c-fn` macro — capture + replace pattern for C function wrapping
- [x] 1.3 Verify both macros compile (load boot.janet headless, check they're in core-env)

## 2. Variadic side-effect wrappers (hide, show, purge, registry-remove)

- [x] 2.1 Convert `hide` — `defn` + `each` + `wrap-c-fn`, remove manual while loop
- [x] 2.2 Convert `show` — same pattern
- [x] 2.3 Convert `purge` — same pattern
- [x] 2.4 Convert `registry-remove` — same pattern
- [x] 2.5 Run `just test-unit` to verify no regressions

## 3. Variadic query wrappers (shape-type, visible?, wire?, face?, solid?)

- [x] 3.1 Convert `shape-type` — `defn` + `seq` + `wrap-c-fn`
- [x] 3.2 Convert `visible?` — same pattern
- [x] 3.3 Convert `wire?` — same pattern
- [x] 3.4 Convert `face?` — same pattern
- [x] 3.5 Convert `solid?` — same pattern
- [x] 3.6 Run `just test-unit`

## 4. Boolean chain wrappers (cut, common, fuse)

- [x] 4.1 Convert `cut` — `defn`, `case` for keyword dispatch, `++` for increments
- [x] 4.2 Convert `common` — same pattern
- [x] 4.3 Convert `fuse` — same pattern
- [x] 4.4 Run `just test-unit`

## 5. Medium wrapper functions (sphere, cone)

- [x] 5.1 Convert `sphere` — `defn`, `case` for keyword dispatch, `++`, `default` for optional radius check
- [x] 5.2 Convert `cone` — same pattern, `seq` for pos-arr collection
- [x] 5.3 Run `just test-unit`

## 6. Complex wrapper functions (box, cylinder, torus)

- [x] 6.1 Convert `box` — `defn`, `case` for keyword dispatch, `++`, `default`
- [x] 6.2 Convert `cylinder` — same pattern, `seq` for pos-arr
- [x] 6.3 Convert `torus` — same pattern, `case` for angle variants (`:a`, `:ar`, `:as`, `:asr`, `:ae`, `:aer`)
- [x] 6.4 Run `just test-unit`

## 7. Transform/extrusion wrappers (extrude, revolve, extrude-polygon, rect, circle, polygon, text, text3d, translate, rotate, scale, mirror)

- [x] 7.1 Convert `extrude` — `defn`, `case` for keyword dispatch
- [x] 7.2 Convert `revolve` — same pattern
- [x] 7.3 Convert `extrude-polygon` — same pattern
- [x] 7.4 Convert `rect` — same pattern
- [x] 7.5 Convert `circle` — same pattern, `default` for radius
- [x] 7.6 Convert `polygon` — same pattern
- [x] 7.7 Convert `text` — same pattern
- [x] 7.8 Convert `text3d` — same pattern
- [x] 7.9 Convert `translate` — same pattern
- [x] 7.10 Convert `rotate` — same pattern, `case` for `:x`/`:y`/`:z`/`:r`/`:a`/`:ar`
- [x] 7.11 Convert `scale` — same pattern
- [x] 7.12 Convert `mirror` — same pattern
- [x] 7.13 Run `just test-unit`

## 8. Thin C-primitive re-exports (edge tools, projection, help, window)

- [x] 8.1 Convert edge visibility toggles (`edge-toggle-inactive`, etc.) — `defn` + `wrap-c-fn`
- [x] 8.2 Convert projection/overlay toggles — same pattern
- [x] 8.3 Convert help window functions — same pattern
- [x] 8.4 Convert window state functions — same pattern
- [x] 8.5 Convert edge styling (`edge-thickness`, `edge-color-inactive`, `edge-color-active`) — same pattern
- [x] 8.6 Run `just test-unit`

## 9. Sketch and wire operation wrappers

- [x] 9.1 Convert sketch wrappers (`sketch`, `move-to`, `line-to`, `line-dx`, `line-dy`, `line-dx-dy`, `arc-to`) — `defn` + `wrap-c-fn`
- [x] 9.2 Convert `close-sketch`, `build-wire`, `wire-to-face`, `wire-fillet`, `wire-chamfer`, `wire-offset` — same pattern
- [x] 9.3 Run `just test-unit`

## 10. I/O wrappers (write-step, write-stl, read-step)

- [x] 10.1 Convert I/O wrappers — `defn` + `wrap-c-fn`, `case` for read-step keywords
- [x] 10.2 Run `just test-unit`

## 11. Selection, queries, and polling

- [x] 11.1 Convert `on-select` — `defn`, `when`/`unless` for conditionals
- [x] 11.2 Convert `poll-selection` — `defn`, `case` for action dispatch
- [x] 11.3 Convert `selected-shapes` — `defn`, `seq`
- [x] 11.4 Convert `list-shapes` — `defn`, `seq`
- [x] 11.5 Convert `list-fonts` — `defn`, `seq`
- [x] 11.6 Run `just test-unit`

## 12. View control and metadata

- [x] 12.1 Convert `view-fit`, `view-fit-all`, `view-angle` — `defn`, `when`
- [x] 12.2 Replace manual metadata `put` calls with `defmeta` for all wrapped functions (~40 calls)
- [x] 12.3 Replace `rojcad-groups` metadata iteration with `each` over groups + `defmeta`
- [x] 12.4 Run `just test-unit`

## 13. display-val, REPL helpers, and my-eval

- [x] 13.1 Convert `display-val` — `defn`, `case` for type dispatch, `seq` for inner loops, `when`
- [x] 13.2 Convert `sort-syms` — `defn`, `for`, `++`
- [x] 13.3 Convert `all-fns`, `apropos`, `cad-fns`, `group` — `defn`, `each over next`
- [x] 13.4 Convert `my-eval` — `defn`, `case` for `(= f0 'def)` / `(= f0 'set)` dispatch
- [x] 13.5 Run `just test-unit`

## 14. Doc generation (gen-markdown, gen-html, split-docstring, dump-docs)

- [x] 14.1 Convert `split-docstring` — `defn`, `for`/`++`
- [x] 14.2 Convert `gen-markdown` — `defn`, `each` for function iteration, `seq` for other-fns
- [x] 14.3 Convert `gen-html` — same pattern
- [x] 14.4 Convert `dump-docs` — `defn`, `try` macro instead of custom `try-catch`
- [x] 14.5 Run `just test-unit`

## 15. View-angle presets: data-driven generation

- [x] 15.1 Design preset data table with `{:front (tuple yaw pitch desc) ...}` format
- [x] 15.2 Replace 7 manual `def view-*` functions with `(each [name yaw pitch desc] view-presets ...)` loop
- [x] 15.3 Replace 7 manual docstring+metadata blocks with `defmeta` inside the generation loop
- [x] 15.4 Remove `defmeta` calls for view-angle presets that are now generated
- [x] 15.5 Run `just test-unit`

## 16. TCP REPL server (connect-handler, listen, accept-loop)

- [x] 16.1 Convert `connect-handler` — `defn`, `try` instead of `try-catch`, `when`
- [x] 16.2 Convert `listen` — `try` instead of `try-catch`
- [x] 16.3 Convert `poll-viewer`, `accept-loop` — `defn`
- [x] 16.4 Run `just test-unit`

## 17. Remove custom try-catch

- [x] 17.1 Replace the custom `try-catch` function definition with upstream `try` at all call sites (~4 uses)
- [x] 17.2 Remove the `try-catch` definition from the top of boot.janet
- [x] 17.3 Run `just test-unit`

## 18. Final verification

- [x] 18.1 Build with `just build` — confirm compilation
- [x] 18.2 Run full test suite with `just test`
- [x] 18.3 Lint with `just lint`
- [x] 18.4 Format check with `just fmt-check`
- [x] 18.5 Manual spot-check: start server, run a few CAD commands in REPL to verify behavior. Note: headless REPL has pre-existing `_poll-selection-raw` nil bug (confirmed in original code, not caused by refactoring). All Rust unit tests (82/82) pass.
