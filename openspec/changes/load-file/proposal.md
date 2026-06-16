## Why

Currently all Janet code in rojcad is embedded at compile time via `include_str!` + `janet_dostring`. The only way to execute Janet code at runtime is over the TCP REPL. There is no way to load code from a file — users cannot define functions, models, or shapes in external `.janet` files and use them in a session. This limits reuse across sessions, sharing of parametric model definitions, and use of third-party Janet CAD scripts.

## What Changes

- Add a `load-file` Janet function that reads a `.janet` file from disk, parses all forms, and evaluates each in the current environment
- `load-file` takes an absolute file path; returns the result of the last form
- Aborts with an error on the first failed form
- Integrates with the existing `my-eval` system for proper shape tracking (auto-purge on redef, auto-show on def)

## Capabilities

### New Capabilities
- `file-loading`: Load and evaluate Janet code from external files at runtime

### Modified Capabilities

<!-- No existing capabilities have requirement changes -->

## Impact

- Only `boot.janet` is affected — no Rust, C, or bridge changes
- New `load-file` function in the `io` category
- No recompile needed — a `just build` suffices
