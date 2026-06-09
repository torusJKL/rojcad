## 1. Replace em dashes in doc strings

- [x] 1.1 Replace em dashes in `boot.janet` doc strings — example comments (`—` → `#`) and prose (`—` → `-`), excluding page titles and file-level comments
- [x] 1.2 Replace em dashes in `bridge/bridge.c` doc strings — example comments (`—` → `#`), leaving C comments unchanged

## 2. Wire upstream `doc-format` into `get-doc`

- [x] 2.1 Modify `get-doc` in `boot.janet` to pass doc strings through `doc-format` before returning

## 3. Fix `pp` to display strings without escaping

- [x] 3.1 Add `pp` override in `boot.janet` that prints strings/buffers with `%s` (raw) instead of `%q` (escaped)

## 4. Verify the change

- [x] 4.1 Build and run headless to confirm no crashes
- [x] 4.2 Connect via spork REPL and verify `(doc box)` shows readable output with proper line breaks and characters
- [x] 4.3 Connect via raw TCP REPL and verify `(doc box)` still works correctly
