## Why

The netrepl port is currently hardcoded to 9365 in `boot.janet`. Users who need a different port (e.g., when the default is occupied, or when running multiple instances) must edit the source and rebuild. A `--port` CLI argument allows runtime configuration without recompilation.

## What Changes

- Add `--port <number>` CLI argument parsing to `main.rs`
- Add `janet_setdyn` to the C bridge so Rust can set dynamic Janet variables
- Set `*netrepl-port*` dynamic variable in the Janet environment before running boot.janet
- Update `boot.janet` to use `(dyn '*netrepl-port*')` with fallback to 9365
- The flag `--headless` is the existing CLI arg; `--port` adds alongside it

## Capabilities

### New Capabilities

- `cli-port-config`: The `--port` CLI argument to override the default netrepl port at runtime

### Modified Capabilities

<!-- No existing specs to modify -->

## Impact

- `src/main.rs`: Parse `--port` argument, set dynamic Janet variable
- `src/bridge.rs`: Add `janet_setdyn` extern declaration
- `boot.janet`: Read `*netrepl-port*` dynamic binding, fall back to 9365
