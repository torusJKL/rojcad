## 1. Fix help window

- [x] 1.1 Update connection info in help.rs: port 9365 → 9364
- [x] 1.2 Update CLI args in help.rs: replace `--port <N>` with `--raw-port <N>` and `--spork-port <N>`
- [x] 1.3 Fix doc syntax in help.rs: `(doc 'sym)` → `(doc sym)`

## 2. Fix integration test

- [x] 2.1 Update test-variadic.sh PORT variable from 9365 to 9364

## 3. Update existing spec

- [x] 3.1 Update openspec/specs/help-window/spec.md: connection port 9365 → 9364, CLI args `--port` → `--raw-port`/`--spork-port`

## 4. Verify

- [x] 4.1 Run `just check` to verify build
- [x] 4.2 Run `cargo test` to verify unit tests (90/90 pass)
- [x] 4.3 Run integration tests (pre-existing failures unrelated)
