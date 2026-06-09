## 1. Vendor spork source files

- [x] 1.1 Download `spork/msg.janet`, `spork/ev-utils.janet` from upstream spork repository; craft server-only `netrepl-server.janet` (getline/rawterm not needed server-side)
- [x] 1.2 Create `vendor/spork/` directory and place the files there (`msg.janet`, `ev-utils.janet`, `netrepl-server.janet`)
- [x] 1.3 Add license attribution for spork (MIT) in `vendor/spork/LICENSE`
- [x] 1.4 Verify vendored files load correctly

## 2. Add CLI flags for port configuration

- [x] 2.1 Add `--spork-port <PORT>` and `--spork-port=<PORT>` parsing in `src/main.rs`
- [x] 2.2 Add `--raw-port <PORT>` and `--raw-port=<PORT>` parsing in `src/main.rs`
- [x] 2.3 Remove old `--port` flag parsing
- [x] 2.4 Set `*spork-repl-port*` and `*raw-repl-port*` Janet dynamic variables based on parsed flags
- [x] 2.5 Validate port ranges (1-65535) for all flags

## 3. Load spork source at boot time

- [x] 3.1 Add `include_str!` for the three spork Janet files in `src/main.rs`
- [x] 3.2 Call `janet_dostring` for each file in dependency order: `msg.janet` → `ev-utils.janet` → `netrepl-server.janet`
- [x] 3.3 Handle load errors gracefully (print to stderr, but don't crash — spork is optional)
- [x] 3.4 Place loading between upstream.janet and boot.janet in the boot sequence

## 4. Update boot.janet for dual servers

- [x] 4.1 Replace `(def port ...)` with `(def raw-port ...)` and `(def spork-port ...)`
- [x] 4.2 Rename existing `listen` to `raw-listen`
- [x] 4.3 Add spork server startup in a dedicated fiber
- [x] 4.4 Wrap spork server startup in error protection
- [x] 4.5 Update the startup banner to print both ports
- [x] 4.6 Update the `(quit)` function (not needed — uses os/exit)

## 5. Update documentation

- [x] 5.1 Update `AGENTS.md`
- [x] 5.2 Update `README.md`
- [x] 5.3 Add Conjure/Neovim instructions to README

## 6. Testing and verification

- [x] 6.1 Verify `just build` compiles with spork source vendored
- [x] 6.2 Verify raw REPL works on port 9364 via `nc 127.0.0.1 9364`
- [x] 6.3 Verify spork REPL works on port 9365 via spork protocol
- [x] 6.4 Verify `--raw-port 9000` changes the raw REPL port
- [x] 6.5 Verify `--spork-port 9001` changes the spork REPL port
- [x] 6.6 Verify both servers accept connections simultaneously
- [x] 6.7 Verify CAD functions work through both REPLs
