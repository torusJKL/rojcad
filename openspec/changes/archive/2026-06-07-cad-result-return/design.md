## Context

Currently, CAD operations in `cad.rs` panic on invalid input via `assert_valid_dimension()` and empty boolean results. These panics propagate through `extern "C"` bridge functions in `main.rs`, which attempt to catch them with `catch_unwind` — but then the error handler itself panics, causing an abort because unwinding through `extern "C"` is undefined behavior.

The result: any invalid user input (e.g., `(box -1)`, `(cut non-intersecting-shapes)`) crashes the entire process instead of returning a recoverable error to the Janet REPL.

27 bridge functions and 18+ CAD functions are affected. 4 wire operations lack `catch_unwind` entirely.

## Goals / Non-Goals

**Goals:**
- All CAD operations validate inputs and return `Result` instead of panicking
- All bridge functions surface errors to Janet via `janet_panic` instead of aborting
- Error messages from CAD operations propagate to the REPL client
- Consistent error-handling pattern across all CAD functions

**Non-Goals:**
- Changing the opencascade-rs dependency or OCCT behavior
- Adding structured error types (String errors are sufficient)
- Changing the Janet-side REPL error recovery (already works once errors reach Janet)
- Performance optimization (error path cost is irrelevant)

## Decisions

### D1: Error type — `String` for all CAD function errors

Use `String` as the error type across all `Result`-returning CAD functions, consistent with the existing `read_step` / `write_step` / `write_stl` pattern.

Rationale: All CAD errors are user-facing messages displayed in the REPL. There's no need for structured error matching on the Rust side — callers either propagate the error or surface it to Janet. A custom enum would add complexity without benefit.

```
Before:                    After:
pub fn make_box(...)        pub fn make_box(...)
    -> ShapeData                -> Result<ShapeData, String>
```

### D2: `validate_dimension` replaces `assert_valid_dimension`

Replace `assert_valid_dimension` (panics on invalid input) with `validate_dimension` (returns `Result<(), String>`).

```rust
fn validate_dimension(value: f64, name: &str) -> Result<(), String> {
    if value <= 0.0 {
        Err(format!("{} must be positive, got {}", name, value))
    } else {
        Ok(())
    }
}
```

All 18 call sites change from `assert_valid_dimension(...)` to `validate_dimension(...)?`.

### D3: Bridge functions return `c_int` instead of panicking

All 27 `rust_init_*` extern "C" functions change from `void` to `c_int`:
- `0` = success
- `1` = error (CAD function returned `Err` or catch_unwind caught an unexpected panic)

The error path does NOT write to `dest` — the allocated Janet abstract memory is left zeroed. This is safe because C will call `janet_panic` which longjmps past any use of the abstract.

```rust
pub unsafe extern "C" fn rust_init_cube(
    dest: *mut c_void, size: c_double, ...
) -> c_int {
    // No catch_unwind for predictable errors
    match cad::make_cube(size, center, eager) {
        Ok(sd) => {
            unsafe { ptr::write(dest as *mut ShapeData, sd); }
            0
        }
        Err(msg) => {
            1  // C caller will janet_panic with the message
        }
    }
}
```

Wait — but how does C get the error message? The bridge functions return only a `c_int` status code. The C code would call `janet_panic("todo")` with a generic message and lose the context.

**Refinement**: Since `janet_panic` is a longjmp (can't safely call from Rust), and we want the error message to propagate, we have two options:

**Option D3a: Return error code, generic C message**
- Rust returns `0`/`1`
- C calls `janet_panic("box: invalid dimensions")` — loses specific values
- Simpler C changes, but worse error messages

**Option D3b: Thread-local error buffer**
- Rust writes error to a thread-local buffer before returning
- C reads the buffer and passes it to `janet_panic`
- Better error messages, more complex

**Option D3c: Return error message via out parameter**
- Rust writes error to a pre-allocated buffer pointer
- C reads the buffer
- Requires size negotiation or fixed-size buffer

**Chosen: D3b — thread-local error buffer**

```rust
std::thread_local! {
    static LAST_CAD_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

pub(crate) fn set_last_error(msg: String) {
    LAST_CAD_ERROR.with(|e| *e.borrow_mut() = msg);
}

pub(crate) fn take_last_error() -> String {
    LAST_CAD_ERROR.with(|e| std::mem::take(&mut *e.borrow_mut()))
}
```

C side:
```c
extern const char *rust_take_last_error(void);
```

Rust:
```rust
pub unsafe extern "C" fn rust_take_last_error() -> *mut c_char {
    let msg = take_last_error();
    CString::new(msg).unwrap().into_raw()
}
```

Then C calls `rust_take_last_error()` when the return code is non-zero, and passes the result to `janet_panic`, then frees the CString.

Rationale: 
- Thread-local storage is safe for our use case (Janet runs on one thread)
- Preserves detailed error messages
- Minimal API surface change (one extra C function)
- Pattern matches existing FFI conventions in the codebase

A simpler alternative: since all `rust_init_*` functions return `c_int`, we could define a single C function `rust_take_last_error()` that returns the error string, and after every `rust_init_*` call that returns non-zero, C calls it.

### D4: `catch_unwind` eliminated for predictable errors

The `catch_unwind` wrapper is removed from bridge functions for predictable errors (input validation, empty boolean results). Since CAD functions now return `Result` instead of panicking, `catch_unwind` is unnecessary for normal operation.

However, `catch_unwind` is **kept as a safety net** for unexpected panics from OCCT internals or Rust bugs. In that case:
- `catch_unwind` catches the panic
- The error handler writes a generic "unexpected error" message to the thread-local buffer
- Returns `1` to C
- C calls `janet_panic("unexpected error in <operation>")`

```rust
pub unsafe extern "C" fn rust_init_cube(...) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| {
        cad::make_cube(size, center, eager)
    }));
    match result {
        Ok(Ok(sd)) => {
            unsafe { ptr::write(dest as *mut ShapeData, sd); }
            0
        }
        Ok(Err(msg)) => {
            set_last_error(msg);
            1
        }
        Err(_panic) => {
            set_last_error("unexpected internal error".to_string());
            1
        }
    }
}
```

### D5: 4 wire operations gain proper error handling

`rust_init_wire_to_face`, `rust_init_wire_fillet`, `rust_init_wire_chamfer`, `rust_init_wire_offset` currently lack any `catch_unwind` and would abort on any panic. They follow the same pattern as D4.

### D6: C bridge pattern

Every `rust_init_*` call in `bridge.c` changes from:

```c
void *shape = alloc_shape();
rust_init_cube(shape, size, ...);
maybe_hide(shape, argv, argc);
return janet_wrap_abstract(shape);
```

To:

```c
void *shape = alloc_shape();
if (rust_init_cube(shape, size, ...)) {
    const char *msg = rust_take_last_error();
    janet_panic(msg);
}
maybe_hide(shape, argv, argc);
return janet_wrap_abstract(shape);
```

The `alloc_shape()` allocation is fine — if `janet_panic` is called, the GC will reclaim the abstract memory.

### D7: Functions excluded from Result migration

These functions don't take user input that can fail and are not called through the bridge:
- `extract_mesh` — already returns `Option`
- `extract_edge_polylines` — no validation, internal
- `generate_synthetic_wireframe` — no validation, internal
- `translate_shape` — mutates in-place, no error possible
- `workplane_from_keyword` — no validation
- `is_wire` / `is_face` / `is_solid` — queries, no error possible
- `SYNTHETIC_WIREFRAME_THRESHOLD` — const

## Risks / Trade-offs

- **Thread-local state** → The error buffer is per-thread, which is safe since Janet runs on a single thread. If threading changes in the future, the buffer must be revisited.
- **Leaked CString** → `rust_take_last_error` returns an owned `*mut c_char`. The C caller must free it with `rust_string_free` (or similar). A helper `janet_panic_with_last_error()` C macro can handle the lifecycle.
- **Zeroed abstract on error path** → If `janet_panic` didn't longjmp (e.g., if someone refactors to not use it), the zeroed ShapeData could be used. This is a C-level invariant on the callers.
- **`catch_unwind` for unexpected panics** → `catch_unwind` has overhead (personality function, landing pads). This only matters on the error path (negligible), and the success path is fast (inline `Result::Ok` match).
- **Validation in two places** → If the C bridge adds its own validation in the future, it could duplicate Rust-side validation. Keep validation in Rust only to avoid drift.
