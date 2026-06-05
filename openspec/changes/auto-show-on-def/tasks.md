## 1. C Bridge — `:hide` keyword helper

- [x] 1.1 Add `maybe_hide(void *data, const Janet *argv, int32_t argc)` static helper in `bridge/bridge.c` that calls `rust_shape_hide(data)` when `:hide` keyword is found in argv

## 2. C Bridge — `:hide` on primitive constructors

- [x] 2.1 Add `maybe_hide` call before each return in `cad_box` (4 return sites)
- [x] 2.2 Add `maybe_hide` call in `cad_sphere` (1 return site)
- [x] 2.3 Add `maybe_hide` call in `cad_cylinder` (3 return sites)
- [x] 2.4 Add `maybe_hide` call in `cad_cone` (1 return site, after `create:`)
- [x] 2.5 Add `maybe_hide` call in `cad_torus` (1 return site, after `create:`)

## 3. C Bridge — `:hide` on boolean and transform constructors

- [x] 3.1 Add `maybe_hide` call in `cad_cut` (1 return site)
- [x] 3.2 Add `maybe_hide` call in `cad_common` (1 return site)
- [x] 3.3 Add `maybe_hide` call in `cad_fuse` (1 return site)
- [x] 3.4 Add `maybe_hide` call in `cad_translate` (1 return site)
- [x] 3.5 Add `maybe_hide` call in `cad_rotate` (1 return site)
- [x] 3.6 Add `maybe_hide` call in `cad_scale` (1 return site)
- [x] 3.7 Add `maybe_hide` call in `cad_mirror` (1 return site)

## 4. C Bridge — Remove `display`

- [x] 4.1 Remove `cad_display` JANET_FN definition from `bridge/bridge.c`
- [x] 4.2 Remove `cad_display_docstring_` and `{"display", cad_display, ...}` from `cfuns` registration array

## 5. Janet — Auto-show on `def` in REPL

- [x] 5.1 Modify `my-eval` in `boot.janet` to inspect top-level form: if it is `(def name expr)` and the result is a shape with `visible?` true, call `(show result)` before returning

## 6. Tests

- [x] 6.1 Build the project with `just build` and verify compilation succeeds
- [x] 6.2 Run `just test-unit` to confirm existing tests pass
- [x] 6.3 Manual REPL test: `(def b (box 10))` → `b` is visible in viewer
- [x] 6.4 Manual REPL test: `(def b (box 10 :hide))` → `b` is NOT visible
- [x] 6.5 Manual REPL test: `(show b)` on hidden shape → `b` becomes visible
- [x] 6.6 Manual REPL test: `(box 10)` alone → shape returned, NOT in viewer
- [x] 6.7 Manual REPL test: `(box 10 :eager :hide)` → both keywords work together
