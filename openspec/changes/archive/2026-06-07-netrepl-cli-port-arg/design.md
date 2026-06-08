## Context

The netrepl port is currently hardcoded in `boot.janet:22` as `(def port 9365)`. The Rust CLI only supports `--headless`. To make the port configurable at runtime without editing source, a `--port` argument needs to flow through to the Janet VM before boot code executes.

## Goals / Non-Goals

**Goals:**
- Accept `--port <number>` or `--port=<number>` from CLI
- Pass the value into Janet as a dynamic variable before running boot.janet
- Fall back to 9365 when `--port` is not provided
- Use the Janet C API (`janet_setdyn`) — no string manipulation of boot code

**Non-Goals:**
- No environment variable support (can be added later)
- No config file support
- No validation beyond basic port range checking

## Decisions

- **Dynamic variables over env table**: Using `janet_setdyn` with a `*netrepl-port*` dynamic binding is the idiomatic Janet approach. It avoids mutating the core environment table and works well with `(dyn)` in boot.janet.
- **`--port <N>` syntax**: Matches the existing `--headless` flag style (no `=` required). Also accepts `--port=<N>` for convenience.
- **Port validation**: Basic rejection of values outside 1–65535 to avoid obvious mistakes. Actual bind failure is handled by `net/listen` in boot.janet.
- **Minimal bridge additions**: Only `janet_setdyn` needs to be added to `bridge.rs`. No new C code or JANET_FN registrations.

## Risks / Trade-offs

- **Invalid port crashes**: If an invalid port is given, the `net/listen` call will fail and the program exits with a message. Acceptable — same as any bad argument.
- **Dynamic var leaks**: Dynamic variables are scoped to the current fiber. Setting `*netrepl-port*` at the top level before running boot.janet ensures it's visible everywhere. No leak risk.
