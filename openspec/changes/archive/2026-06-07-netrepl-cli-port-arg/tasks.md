## 1. Bridge Layer

- [x] 1.1 Add `janet_setdyn` extern declaration to `src/bridge.rs`

## 2. Rust CLI Parsing

- [x] 2.1 Parse `--port <number>` and `--port=<number>` arguments in `src/main.rs`
- [x] 2.2 Validate port is in range 1–65535, print error and exit if invalid
- [x] 2.3 Call `janet_setdyn` to set `*netrepl-port*` before running boot.janet

## 3. Janet Boot Code

- [x] 3.1 Update `boot.janet` to use `(dyn '*netrepl-port*')` with fallback to 9365

## 4. Verify

- [x] 4.1 Build with `just build` to confirm no errors
- [x] 4.2 Run tests with `just test` to confirm nothing is broken
