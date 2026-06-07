## Why

A single evaluation error in the Janet TCP REPL (e.g., calling `show` with wrong arity) kills the connection handler fiber, leaving the TCP socket open but unresponsive. The client hangs forever with no way to recover except reconnecting.

## What Changes

- Fix `try-catch` to actually catch errors (currently errors propagate through `resume` because fibers lack the `:e` error-protection flag)
- Wrap `my-eval` call in `try-catch` so evaluation errors are returned to the client as strings instead of killing the REPL fiber
- Change `ev/go` call to pass a function instead of a pre-built fiber, so error masking is applied automatically by the scheduler

## Capabilities

### New Capabilities

- `repl-error-recovery`: Graceful error handling in the Janet TCP REPL — evaluation errors are caught, reported to the client as error strings, and the REPL continues accepting commands

### Modified Capabilities

None — no existing specs to modify.

## Impact

- `boot.janet` only (the REPL server code)
- No Rust code, no C bridge code, no viewer code affected
- No external dependencies changed
