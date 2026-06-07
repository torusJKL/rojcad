## 1. Janet — Auto-purge in my-eval

- [x] 1.1 Capture old binding from `core-env` before `compile`/`resume` when the form is a top-level `def` or `set`
- [x] 1.2 Purge the old value after successful evaluation if it was a `:rojcad/shape` and is not the same object as the result (identity guard via `not=`)
- [x] 1.3 Extend the auto-show condition to also fire on `set` forms (change `(= f0 'def)` to `(or (= f0 'def) (= f0 'set))`)

## 2. Build and Test

- [x] 2.1 Build the project with `just build` and verify compilation succeeds
- [x] 2.2 Run `just test-unit` to confirm existing tests pass
- [x] 2.3 Manual REPL test: `(def obj (box 10)) (def obj (sphere 20))` — box gone, sphere visible
- [x] 2.4 Manual REPL test: `(var obj (box 10)) (set obj (sphere 20))` — box purged, sphere auto-shown (note: `def` creates constants, `set` requires `var`)
- [x] 2.5 Manual REPL test: `(def obj (box 10)) (def obj obj)` — no panic, shape stays visible
- [x] 2.6 Manual REPL test: `(def a (box 10)) (def b a) (def a (sphere 20))` — shared ref, `(show b)` errors
- [x] 2.7 Manual REPL test: `(def obj (box 10)) (def obj (bad-fn))` — error preserves old binding
