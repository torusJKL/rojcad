## Why

The help window (help.rs) and its spec were never updated when the spork-repl change split the single REPL into two servers (raw TCP on 9364, spork netrepl on 9365). The help window still shows port 9365 for `nc`, references the removed `--port` flag, and uses the wrong `(doc 'sym)` syntax. The integration test also connects `nc` to the wrong port (9365 instead of 9364).

## What Changes

- **help.rs**: Fix three stale references in the help overlay
  - `nc 127.0.0.1 9365` → `nc 127.0.0.1 9364`
  - `--port <N>` → `--raw-port <N>` and `--spork-port <N>`
  - `(doc 'sym)` → `(doc sym)`
- **test-variadic.sh**: Fix TCP REPL test port from 9365 to 9364
- **help-window/spec.md**: Update connection info and CLI args scenarios to match current behavior

No breaking changes — these are fixes to stale documentation that already describes incorrect behavior.

## Capabilities

### New Capabilities

*(none — this is a fix, not a new capability)*

### Modified Capabilities

- `help-window`: Connection info scenario shows wrong port (9365 instead of 9364); CLI args scenario lists `--port` which no longer exists (replaced by `--raw-port`/`--spork-port`)

## Impact

| File | Change |
|------|--------|
| `src/viewer/help.rs` | 3 edits: port in nc command, port in descriptive text, CLI flags, doc syntax |
| `tests/test-variadic.sh` | 1 edit: PORT variable 9365→9364 |
| `openspec/specs/help-window/spec.md` | 2 edits: connection port, CLI args list |
