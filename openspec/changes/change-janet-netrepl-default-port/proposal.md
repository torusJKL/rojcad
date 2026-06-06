## Why

Port 9000 is commonly used by other development tools, causing conflicts. 9365 is the designated IANA-registered port for Janet, making it the correct default for a Janet netrepl server.

## What Changes

- Change the default netrepl port in `boot.janet` from 9000 to 9365
- Update the doc comment in `src/main.rs` that references port 9000
- The new port is a **BREAKING** change for anyone connecting to port 9000

## Capabilities

### New Capabilities

- `netrepl-port`: The default TCP port for the Janet netrepl server, configurable at build time

### Modified Capabilities

<!-- No existing specs to modify -->

## Impact

- `boot.janet`: `(def port 9000)` → `(def port 9365)`
- `src/main.rs`: doc comment update
- Users connecting to the REPL must now use port 9365
