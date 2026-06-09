## 1. Fetch upstream boot.janet

- [x] 1.1 Fetch upstream Janet `src/boot/boot.janet` from GitHub matching the vendored version (v1.41.2 from `vendor/janetconf.h`) and save as `upstream.janet` at project root

## 2. Modify build initialization

- [x] 2.1 Add `include_str!("../upstream.janet")` and a `janet_dostring` call before the existing boot.janet load in `src/main.rs`

- [x] 2.2 Update the error handling and boot order comment to reflect the two-phase loading (upstream macros first, then rojcad boot code)

## 3. Update documentation

- [x] 3.1 Update AGENTS.md to document that standard Janet macros are now available via `upstream.janet`, and note the remaining `&form`/`&env` limitation

## 4. Verify

- [x] 4.1 Build the project with `just build` and confirm it compiles without errors

- [x] 4.2 Run unit tests with `just test-unit` to ensure no regressions

- [x] 4.3 Start the server and manually verify that `(defmacro twice [x] (tuple '+ x x))` followed by `(twice 5)` returns `10` in the REPL
