## Why

The application crashes with a silent `Segmentation fault (core dumped)` — no stack trace, no error message, no crash log. Debugging is nearly impossible: you don't know which thread crashed, which operation triggered it, or where in the code it happened. Rust panics inside `catch_unwind` blocks are also silently swallowed and replaced with generic "unexpected error" messages. Meanwhile, calling `janet_panic` directly from Rust code is undefined behavior (`longjmp` past live Rust frames).

Add a crash diagnostics system that captures SIGSEGV/panics with resolved backtraces, surfaces real panic messages to the user, and fixes the `longjmp` safety issue.

## What Changes

- **New `crash_handler` module**: Install a `SIGSEGV` / `SIGABRT` / `SIGBUS` / `SIGILL` signal handler at startup that writes a resolved backtrace to a crash log file before exiting
- **Custom panic hook**: Replaces the default hook, logs panics (with forced backtrace capture) to the same crash log file
- **Better `catch_unwind` messages**: Extract and forward real panic messages instead of generic "unexpected error" strings — errors appear in the REPL with actionable detail
- **Helper functions for `catch_unwind`**: 36 near-identical `catch_unwind` blocks collapsed into reusable helpers (`catch_cad`, `catch_result`)
- **Fix `janet_panic` UB from Rust**: The two GUI REPL FFI functions that call `janet_panic` from Rust `extern "C"` frames are moved to C JANET_FN wrappers in `bridge.c`, eliminating undefined behavior
- **Crash log file**: Written to the system temp directory (Linux: `/tmp`, macOS: `/tmp`, Windows: `%TEMP%`) — persists only on crash; removed on clean exit
- **Dependencies**: Add `backtrace` and `libc` crates

## Capabilities

### New Capabilities
- `crash-diagnostics`: Signal handler installation, panic hook, crash log file management, backtrace capture and resolution

### Modified Capabilities

None — this is purely additive, with no spec-level behavior changes to existing capabilities.

## Impact

- **New dependencies**: `backtrace`, `libc`
- **`src/crash_handler.rs`**: New module (~300 lines)
- **`src/main.rs`**: Add `mod crash_handler` and one call to `crash_handler::init()` at top of `main()`. Replace 36 `catch_unwind` blocks with helper calls. Remove direct `bridge::janet_panic` calls from GUI REPL FFI functions
- **`bridge/bridge.c`**: Add JANET_FN wrappers for GUI REPL poll/send functions
- **Testing**: New unit tests for panic hook formatting, helper functions, and crash file path. Existing REPL test suite continues to pass (improved error messages may change expected strings in tests)
