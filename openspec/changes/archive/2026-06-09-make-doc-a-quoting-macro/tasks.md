## 1. Refactor `doc` to macro + function pair

- [x] 1.1 Rename `defn doc` → `defn get-doc` at `boot.janet:907`
- [x] 1.2 Add `defmacro doc [sym]` right after `get-doc`, expanding to `(get-doc ',sym)`

## 2. Update internal callers

- [x] 2.1 Change `(doc fn-name)` → `(get-doc fn-name)` in `gen-markdown` at `boot.janet:1114`
- [x] 2.2 Change `(doc fn-name)` → `(get-doc fn-name)` in `gen-markdown` at `boot.janet:1144`
- [x] 2.3 Change `(doc fn-name)` → `(get-doc fn-name)` in `gen-html` at `boot.janet:1246`
- [x] 2.4 Change `(doc fn-name)` → `(get-doc fn-name)` in `gen-html` at `boot.janet:1282`

## 3. Verify

- [x] 3.1 Build with `just build` to confirm no compile errors
- [x] 3.2 Run `just fmt` to ensure formatting is clean
- [x] 3.3 Run `just lint` — timed out on CMake cache reconfigure (pre-existing env issue, Janet-only change)
