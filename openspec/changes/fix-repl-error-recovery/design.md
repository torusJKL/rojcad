## Context

The Janet TCP REPL server (`boot.janet`) uses a fiber-based event loop with `ev/go`. When a client sends an expression, it is parsed, evaluated, and the result is written back. Currently, errors from user code propagate uncaught through the fiber call chain, killing the connection handler fiber permanently.

The root cause is two-layered:

1. **`resume` propagates errors by default** — `fiber/new` without `:e` flag creates fibers without error masking, so `(resume f)` passes `JANET_SIGNAL_ERROR` up to the parent fiber rather than returning the error value.
2. **No error boundary around evaluation** — `my-eval` on line 48 is called directly without any error wrapping, so any panic in user code kills the entire `connect-handler` fiber.

## Goals / Non-Goals

**Goals:**
- Every evaluation error is caught and returned to the client as an error string
- The REPL continues accepting commands after any error
- The TCP connection is closed cleanly on EOF or read error
- Zero changes to Rust or C bridge code

**Non-Goals:**
- Error recovery from infinite loops or `(os/exit)` (these still kill the fiber)
- Changing the wire protocol or response format
- Adding supervisor channels or monitoring

## Decisions

**Decision 1: Fix `try-catch` to use `:e` flag**

The existing `try-catch` macro creates fibers without error protection:
```janet
(def f (fiber/new body))  ← error propagates through resume
```

Change to:
```janet
(def f (fiber/new body :e))  ← error is caught, resume returns error value
```

This is a one-character fix and makes `try-catch` actually work.

**Alternatives considered:**
- Reimplement using `protect` / `pcall` — Janet doesn't have these built-in; `try-catch` IS our pcall.
- Manual `fiber/status` check without `:e` flag — doesn't work because the error propagates before you can check.

**Decision 2: Wrap `my-eval` in `try-catch`**

```janet
(def eval-result (try-catch (fn [] (my-eval parsed env)) (fn [e] e)))
```

This ensures any panic from `my-eval` (compilation errors, runtime errors) is caught and returned to the client as a string. The REPL loop continues.

**Decision 3: Pass function to `ev/go` instead of pre-built fiber**

Current:
```janet
(ev/go (fiber/new (fn [] (connect-handler conn))))
```

Changed to:
```janet
(ev/go (fn [] (connect-handler conn)))
```

When `ev/go` receives a function, it creates the fiber with error masking (`JANET_FIBER_MASK_ERROR | JANET_FIBER_MASK_USER0-4`), providing a second layer of defense.

## Risks / Trade-offs

- **No change to stack trace quality** — Errors will be returned as their string representation (e.g., `"error: arity mismatch, expected at most 1, got 2"`). The stack trace is still printed on stderr server-side via `janet_stacktrace_ext` in the event loop when the fiber eventually dies, but since errors are now caught, this shouldn't trigger.
- **`try-catch` still doesn't protect against Janet VM panics** — C-level panics (segfaults, OOM) are not recoverable. This is expected and acceptable.
- **Risk of masking bugs** — Wrapping everything in try-catch could theoretically swallow unexpected errors. Mitigated by returning the error string to the client, making them visible.
