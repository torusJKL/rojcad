## 1. Add `parametric-models` category to `cad-groups`

- [x] 1.1 Add `"parametric-models" "Parametric Models"` entry to `cad-groups` table in `boot.janet`

## 2. Add `setmeta` calls to `boot/model.janet`

- [x] 2.1 Add `setmeta` call for `defmodel` macro with category `"parametric-models"` and docstring describing macro syntax including `:parts` and `:result` keywords
- [x] 2.2 Add `setmeta` call for `build` function with usage, body, examples, and return type
- [x] 2.3 Add `setmeta` call for `graph` function with usage, body, examples, and return type
- [x] 2.4 Add `setmeta` call for `highlight` function with usage, body, examples, and return type
- [x] 2.5 Add `setmeta` call for `highlight-clear` function with usage, body, examples, and return type

## 3. Add Parametric Models section to README

- [x] 3.1 Add "Parametric Models" section to README.md with workflow narrative example showing `defmodel`, `build`, `graph`, `highlight`, and rebuild

## 4. Verify documentation output

- [x] 4.1 Run `just doc-janet` and verify `defmodel`, `build`, `graph`, `highlight`, `highlight-clear` appear in `doc/janet-api.md` under "Parametric Models"
- [x] 4.2 Verify each entry has correct usage, examples, and return type
- [x] 4.3 Verify README reads well and the example is self-consistent
