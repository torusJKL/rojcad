## 1. Shared State

- [x] 1.1 Add `use std::sync::OnceLock;` and `pub static HELP_EXAMPLE: OnceLock<String> = OnceLock::new();` to `src/types.rs`

## 2. Rust FFI Bridge

- [x] 2.1 Import `HELP_EXAMPLE` and `CStr` in `src/main.rs`
- [x] 2.2 Add `#[unsafe(no_mangle)] pub unsafe extern "C" fn rust_help_set_example(path: *const c_char)` FFI function in `src/main.rs` that converts the C string and stores it in `HELP_EXAMPLE`

## 3. C Bridge

- [x] 3.1 Add `extern void rust_help_set_example(const char *path);` forward declaration in `bridge/bridge.c`
- [x] 3.2 Add `JANET_FN(cad_help_set_example, ...)` function in `bridge/bridge.c` that extracts the string via `janet_getstring` and calls `rust_help_set_example`
- [x] 3.3 Register `{"help-set-example", cad_help_set_example, cad_help_set_example_docstring_}` in the `cad_register_functions` table in `bridge/bridge.c`

## 4. Help Window UI

- [x] 4.1 Import `HELP_EXAMPLE` from `crate::types` in `src/viewer/help.rs`
- [x] 4.2 Add a "Quick Example" section in `Help::ui()` after "Connecting to REPL" and before "Command Line Arguments", gated on `HELP_EXAMPLE.get()`, rendering the expression in `ui.monospace()` with the description "Export all visible shapes to a STEP file"

## 5. Janet Registration

- [x] 5.1 Append OS check and `help-set-example` call at the end of `boot.janet` — using `(os)` to select `/tmp/model.step` (Unix) or `C:\temp\model.step` (Windows), with the example `(def mybox (box 10))\n(write-step ...)`

## 6. Spec Update

- [x] 6.1 Apply the delta spec to `openspec/specs/help-window/spec.md` — add the "Quick Example section" and "help-set-example function" requirements from the change's delta spec
