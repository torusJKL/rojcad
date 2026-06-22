## Context

The application currently has zero crash handling infrastructure. A `SIGSEGV` from the OCCT C++ kernel kills the process silently with `Segmentation fault (core dumped)`. Rust panics inside `catch_unwind` blocks are swallowed — the actual panic message is discarded and replaced with generic `"unexpected error in rust_init_box"`. Two functions in `main.rs` call `janet_panic` directly from Rust `extern "C"` frames, which is undefined behavior because `janet_panic` performs a `longjmp` past live Rust stack frames.

The existing codebase has 36 near-identical `catch_unwind` blocks in `src/main.rs`, each following the same pattern: `Ok(Ok) → success`, `Ok(Err) → set_last_error`, `Err(_) → set_last_error("unexpected error ...")`.

This design adds a `crash_handler` module for signal handling and panic hooking, refactors the `catch_unwind` pattern, and fixes the `longjmp` UB in the GUI REPL FFI.

## Goals / Non-Goals

**Goals:**
- Capture SIGSEGV/SIGABRT/SIGBUS/SIGILL with a resolved backtrace written to a crash log file in `std::env::temp_dir()`
- Capture Rust panics (regardless of `RUST_BACKTRACE` env) with message + backtrace in the same crash log file
- Surface real panic messages in the REPL instead of generic "unexpected error" strings
- Collapse 36 repetitive `catch_unwind` blocks into reusable helpers
- Fix the defined behavior: `janet_panic` longjmp from Rust frames in GUI REPL FFI
- Remove crash log file on clean exit (no crash = no litter)

**Non-Goals:**
- Viewer thread health monitoring or graceful shutdown of the GPU pipeline
- Crash report submission or telemetry
- minidump / coredump generation (OS core dumps still work independently)
- Windows Structured Exception Handling (SEH) as an alternative to POSIX signals — the `backtrace` crate provides cross-platform abstraction

## Decisions

### Decision 1: Signal handler architecture — `fork()` for symbol resolution

Signal handlers must only call async-signal-safe functions. Symbol resolution (`backtrace::resolve`, `dladdr`) calls `malloc` internally, which is NOT signal-safe.

**Approach**: In the signal handler, use `backtrace::trace()` to capture raw addresses into a stack-allocated fixed-size array (`[usize; 128]`). Then `fork()` a child process — `fork()` is async-signal-safe. The child has a clean address space mirroring the parent and can safely call `backtrace::resolve()` and write to the crash file. The parent calls `waitpid(child, WNOHANG)` then `_exit(128 + sig)`.

**Alternatives considered**:
- *Pre-resolve all possible frames at startup*: Impractical — we don't know what frames will be on the stack during a crash
- *Write raw addresses, resolve on next launch*: Complex state management across sessions, and the binary might change between crashes
- *Use `backtrace_symbols_fd` (glibc)*: Not portable to macOS/Windows, requires `libc` crate anyway

### Decision 2: Pre-open crash log file at startup

Opening a file inside a signal handler is NOT async-signal-safe. Instead, the crash log file is created at startup and the `RawFd` is stored in a `OnceLock`. The signal handler writes to this pre-opened fd using `libc::write()` (async-signal-safe).

On clean exit (`main()` returns normally), the file is removed via a `Drop` guard or `atexit` handler. On crash, the file persists with crash data.

### Decision 3: Helper functions to replace 36 catch_unwind blocks

We create two helpers that encapsulate the repetitive `catch_unwind` pattern:

```rust
/// For operations that produce a ShapeData (box, sphere, translate, etc.)
fn catch_cad<F>(name: &str, dest: *mut c_void, f: F) -> c_int
where F: FnOnce() -> Result<ShapeData, String>;

/// For operations that produce no value (hide, write-step, etc.)
fn catch_result<F>(name: &str, f: F) -> c_int
where F: FnOnce() -> Result<(), String>;
```

Each helper internally calls `catch_unwind(AssertUnwindSafe(f))`, handles `Ok(Ok)`, `Ok(Err)`, and `Err(panic_download)`, and on panic extracts the real message via `panic.downcast_ref::<String>()` or `panic.downcast_ref::<&str>()`.

Pro: One change propagates across all 36 call sites (e.g., if we add another logging step).
Con: The helper signature slightly constrains the call sites (must return `Result<ShapeData, String>` or `Result<(), String>`).

### Decision 4: Fix `janet_panic` UB by moving GUI REPL to bridge.c

The two functions `janet_gui_repl_poll_request` and `janet_gui_repl_send_response` (inline `unsafe extern "C"` in `main.rs:2363-2396`) call `bridge::janet_panic` from Rust frames. This is UB — `janet_panic` does a `longjmp` past the Rust frame without running destructors.

**Fix**: Move the wrapper logic to `bridge.c` as proper JANET_FN functions. The Rust side exposes simple data functions (`rust_gui_repl_poll_request_at` returns `*mut c_char` or null, `rust_gui_repl_send_response_at` returns nothing). The C JANET_FN wrappers call these for data and call `janet_panic` from C context (safe).

**Alternative considered**: Restructure the inline Rust functions to avoid calling `janet_panic` directly. This would require changing the return convention (signal error via return value), but then the caller (Janet VM) wouldn't know about the error. JANET_FN is the intended mechanism for signaling errors to Janet.

### Decision 5: Custom panic hook

The panic hook is called BEFORE `catch_unwind` catches the panic (if at all). This gives us a guaranteed log of every panic regardless of whether `catch_unwind` swallows it.

The hook:
- Extracts file:line from `PanicInfo::location()`
- Downcasts payload for `String` or `&str`
- Captures `Backtrace::capture()` (always — ignores `RUST_BACKTRACE`)
- Writes formatted report to crash file + stderr
- Does NOT suppress the default behavior (we'd replicate it)

### Decision 6: `CAD_CHECK` macro left unchanged

After code review: the `CAD_CHECK` macro calls `janet_panic` from C code (the JANET_FN in `bridge.c`), AFTER the Rust `catch_unwind` has returned. There are no Rust frames on the stack when `janet_panic` is called. No UB — no change needed.

### Decision 7: Cross-platform handling

The `backtrace` crate abstracts platform differences for stack capture and symbol resolution. On Linux, it uses `libunwind` + DWARF. On macOS, it uses `libunwind` + Mach-O symbols. On Windows, it uses `CaptureStackBackTrace` + dbghelp.

For the `sigaction` installation, we use `libc::sigaction` on Unix and skip the signal handler on Windows (Windows uses `SetUnhandledExceptionFilter` via the `backtrace` crate's Minidump support — but that's a separate concern). For this initial implementation, the signal handler is Unix-only; the panic hook and crash log file work on all platforms.

## Risks / Trade-offs

- **[Fork in signal handler]**: `fork()` can fail (resource limits). If fork fails, we fall back to writing raw addresses to the crash file fd and exiting. The raw addresses can be resolved offline with `addr2line`.
- **[Stack overflow]**: SIGSEGV from stack overflow means even stack-allocated buffers may fail. We mitigate by setting up `sigaltstack` at startup.
- **[Deadlock in panic hook]**: If the panic hook itself panics (e.g., crash file write fails), Rust aborts the process. The signal handler fd write is still available and will contain whatever was written before the hook panic.
- **[Performance]**: The panic hook runs for every panic, including panics caught by `catch_unwind`. The overhead of `Backtrace::capture()` is non-trivial (~100µs). Acceptable — panics are exceptional, not on hot paths.
- **[LTO symbol quality]**: Release builds with LTO may lose some frame name information. The `backtrace` crate with DWARF debug info still produces good results. Debug info stripping in release builds would limit quality — we can document that users should keep debug info or use a separate debug package.
- **[Temp directory write failure]**: If the temp directory is not writable, crash file creation fails at startup. The `crash_handler::init()` logs a warning to stderr and continues without crash file support (panic hook and signal handler still write to stderr).
