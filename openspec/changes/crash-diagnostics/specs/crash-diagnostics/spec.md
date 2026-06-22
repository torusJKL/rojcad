## ADDED Requirements

### Requirement: Signal handler captures SIGSEGV

The system SHALL install a signal handler at startup for SIGSEGV, SIGABRT, SIGBUS, and SIGILL. When a signal is received, the system SHALL capture a backtrace with resolved file:line symbols, write a crash report to a file in the system temp directory, and exit with code 128 + signal number.

#### Scenario: SIGSEGV during CAD operation
- **WHEN** the OCCT C++ kernel triggers a SIGSEGV during a translate operation
- **THEN** `std::env::temp_dir() / "rojcad_crash_{pid}.log"` SHALL contain: signal name, fault address, thread name, and resolved backtrace showing the call chain from `janet_call` through the JANET_FN wrapper to the OCCT function

#### Scenario: SIGABRT during boot
- **WHEN** a SIGABRT is raised during Janet initialization
- **THEN** the signal handler SHALL write a crash report and exit with code 134 (128+6)

### Requirement: Alternate signal stack for stack overflow

The system SHALL set up an alternate signal stack via `sigaltstack` before installing signal handlers, so that a stack-overflow-induced SIGSEGV can be handled rather than causing a double fault.

#### Scenario: Stack overflow recovery
- **WHEN** a deep recursive Janet call exhausts the main stack, triggering SIGSEGV
- **THEN** the signal handler SHALL run on the alternate stack and produce a crash report

### Requirement: Crash log file at startup

The system SHALL open a crash log file at `std::env::temp_dir() / "rojcad_crash_{pid}.log"` during startup. The file SHALL be removed on clean exit and SHALL persist on crash.

#### Scenario: File creation on startup
- **WHEN** `crash_handler::init()` is called
- **THEN** the crash log file SHALL be created and opened for writing

#### Scenario: File removed on clean exit
- **WHEN** the application exits normally
- **THEN** the crash log file SHALL be deleted

#### Scenario: File creation failure warning
- **WHEN** the temp directory is not writable
- **THEN** the system SHALL log a non-fatal warning to stderr and continue without crash file support

### Requirement: Panic hook logs all Rust panics

The system SHALL replace the default Rust panic hook. The custom hook SHALL always capture a backtrace (regardless of `RUST_BACKTRACE` environment variable), extract the panic message and source location, write a formatted report to the crash log file and stderr.

#### Scenario: Panic from CAD operation
- **WHEN** `cad::make_box` panics with `"index out of bounds"` and `catch_unwind` catches it
- **THEN** the panic hook SHALL write `"PANIC\n  at src/cad.rs:42\n  index out of bounds\nBacktrace:\n  0: ..."` to the crash log file BEFORE the catch_unwind handler runs

### Requirement: Real crash messages propagate to user

The system SHALL extract the actual panic message in `catch_unwind` blocks and forward it through the existing Janet error mechanism, so the user sees actionable error messages in the REPL instead of generic "unexpected error" text.

#### Scenario: REPL shows panic detail
- **WHEN** a CAD function panics (e.g., unwrap on an Err)
- **THEN** the REPL SHALL display `"rust_init_box panicked: called 'Result::unwrap()' on an 'Err' value: ..."` instead of `"unexpected error in rust_init_box"`

### Requirement: Helper functions for catch_unwind

The system SHALL provide `catch_cad(name, dest, f)` and `catch_result(name, f)` helper functions that encapsulate the `catch_unwind` pattern, reducing 36 repeated blocks to a single implementation.

#### Scenario: catch_cad returns ShapeData
- **WHEN** the inner function returns `Ok(ShapeData{shape_id: 5, ...})`
- **THEN** `catch_cad` SHALL write the ShapeData to `dest`, register the shape pointer, and return 0

#### Scenario: catch_cad forwards expected errors
- **WHEN** the inner function returns `Err("invalid dimension")`
- **THEN** `catch_cad` SHALL call `set_last_error("invalid dimension")` and return 1

#### Scenario: catch_cad captures panic messages
- **WHEN** the inner function panics with `"division by zero"`
- **THEN** `catch_cad` SHALL call `set_last_error("rust_init_box panicked: division by zero")` and return 1

### Requirement: GUI REPL FFI does not longjmp from Rust frames

The system SHALL NOT call `janet_panic` from Rust `extern "C"` functions. The two GUI REPL FFI functions SHALL be restructured so that `janet_panic` is only called from C code (JANET_FN wrappers in `bridge.c`).

#### Scenario: REPL poll without pending request
- **WHEN** `rust_gui_repl_poll_request_at()` is called and no request is pending
- **THEN** it SHALL return NULL, and the C JANET_FN wrapper SHALL return `janet_wrap_nil()` (no panic)

#### Scenario: REPL send with wrong argument type
- **WHEN** `gui-repl-send-response` is called with a non-string argument
- **THEN** the C JANET_FN wrapper SHALL call `janet_panic("expected string")` from C context

### Requirement: Windows support via platform abstraction

The system SHALL use `std::env::temp_dir()` for crash file location (resolves to `%TEMP%` on Windows), and the `backtrace` crate SHALL provide cross-platform backtrace capture. The POSIX signal handler SHALL be conditionally compiled only on Unix targets.

#### Scenario: Crash report on Windows
- **WHEN** a Rust panic occurs on Windows
- **THEN** the panic hook SHALL write a crash report to `%TEMP%\rojcad_crash_{pid}.log` with a backtrace captured via Windows `CaptureStackBackTrace`
