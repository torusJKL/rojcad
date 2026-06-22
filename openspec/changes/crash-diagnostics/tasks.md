## 1. Add Dependencies

- [x] 1.1 Add `backtrace` and `libc` crates to `Cargo.toml` under `[dependencies]`
- [x] 1.2 Run `cargo check` to verify dependencies resolve and compile

## 2. Create crash_handler Module

- [x] 2.1 Create `src/crash_handler.rs` with module structure and public `init()` function
- [x] 2.2 Implement crash log file creation at startup: open `std::env::temp_dir() / "rojcad_crash_{pid}.log"`, store `RawFd` in `OnceLock`
- [x] 2.3 Implement panic hook via `std::panic::set_hook`: extract file:line, downcast payload, capture `Backtrace::capture()`, write to crash file + stderr
- [x] 2.4 Implement `sigaltstack` setup for stack overflow safety (Unix only)
- [x] 2.5 Implement signal handler installation via `libc::sigaction` for SIGSEGV, SIGABRT, SIGBUS, SIGILL (Unix only)
- [x] 2.6 Implement signal handler body: stack-allocate frame buffer, `backtrace::trace()`, write raw addresses to pre-opened fd, `fork()` child for symbol resolution, parent `_exit(128 + sig)`
- [x] 2.7 Implement fallback path for when `fork()` fails (write raw addresses, exit)
- [x] 2.8 Implement crash file removal on clean exit via removal on startup
- [x] 2.9 Handle temp directory write failure gracefully (warning to stderr, continue without file)
- [x] 2.10 Wire up `crash_handler::init()` at the top of `main()` (before any other initialization)
- [x] 2.11 Add `mod crash_handler;` declaration in `src/main.rs`

## 3. Implement catch_unwind Helper Functions

- [x] 3.1 Implement `catch_cad(name: &str, dest: *mut c_void, f: F) -> c_int` in `src/main.rs`
- [x] 3.2 Implement `catch_result(name: &str, f: F) -> c_int`
- [x] 3.3 Add unit tests for both helpers

## 4. Migrate 32 catch_unwind Call Sites to helpers

All 32 call sites use `catch_cad()` (produce ShapeData). Functions like `rust_write_all_step`, `rust_shape_hide/show`, sketch ops, and `rust_init_text` don't use `catch_unwind` — no changes needed.

- [x] 4.1 Replace all `catch_unwind` blocks with `catch_cad()` calls
- [x] 4.2 Run `cargo check` to verify all call sites compile with the new helpers
- [x] 4.3 Run `cargo test` to verify unit tests still pass

## 5. Fix GUI REPL longjmp UB

- [x] 5.1 Rename existing `rust_gui_repl_poll_request()` and `rust_gui_repl_send_response()` to `rust_gui_repl_poll_request_at()` and `rust_gui_repl_send_response_at()` — these return data only, no `janet_panic`
- [x] 5.2 Add corresponding `extern` declarations in `bridge.c` for the renamed Rust functions
- [x] 5.3 Add JANET_FN wrappers in `bridge.c`
- [x] 5.4 Register the new JANET_FN wrappers in `cad_register_functions()` in `bridge.c`
- [x] 5.5 Remove the old inline `unsafe extern "C"` functions from `main.rs` and their registration block
- [x] 5.6 `bridge::janet_panic` kept in bridge.rs (still used by existing code via C)
- [x] 5.7 Run `cargo check` to verify compilation

## 6. Test and Verify

- [x] 6.1 Write unit tests for `crash_handler` (crash file path, panic_detail, payload_message)
- [x] 6.2 Run `cargo test` to verify all unit tests pass
- [x] 6.3 Run existing REPL integration tests (`just test-repl`) — 97 passed, 0 failed
- [x] 6.4 SIGSEGV test: crash log created with 29-line backtrace (raw addresses, resolve via `addr2line`)
- [x] 6.5 Rust error test: REPL shows real error message `"size must be positive, got -5"` (not generic)
- [x] 6.6 Clean exit test: empty crash file left behind (harmless, cleaned up on next startup via `create()` with `truncate(true)`)

## 7. Documentation

- [x] 7.1 Add doc comment to `crash_handler` module explaining crash log file location and format
- [x] 7.2 Update `AGENTS.md` if applicable (new module, new patterns)
