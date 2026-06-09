## Why

The current raw TCP REPL (`nc 127.0.0.1 9365`) provides no line editing, history, multi-line input, syntax highlighting, or completion. Switching to the spork netrepl protocol unlocks Conjure (Neovim) and the spork CLI client as full-featured REPL frontends with SLY-like capabilities.

## What Changes

- **Add spork as a runtime Janet dependency** — load `spork/netrepl` at boot time alongside the existing REPL server
- **Add a spork netrepl server** on port 9365 — the default port that all spork clients (Conjure, spork CLI, Freja-netrepl) expect
- **Move the raw TCP REPL to port 9364** — preserves `nc` compatibility with a one-digit port change
- **Add `--spork-port` and `--raw-port` CLI flags** for overriding defaults; replace the single `--port` flag
- **Update the startup banner** to print both ports and their intended clients
- **Update docs** (README, doc/janet-api.md) to reflect the new port layout
- **Remove the old `--port` flag** — use `--raw-port` and `--spork-port` instead

## Capabilities

### New Capabilities

- `spork-repl`: Enable the spork/netrepl protocol server for client connections on port 9365

### Modified Capabilities

- `netrepl-port`: The default raw REPL port changes from 9365 to 9364. The `*netrepl-port*` dynamic variable continues to control the raw REPL port (now 9364 by default).

## Impact

- **New dependency**: spork source files must be vendored or embedded for bootstrapped Janet
- **boot.janet**: Add a second TCP server alongside the existing one; both run concurrently on different ports
- **src/main.rs**: Replace `--port` with `--spork-port` and `--raw-port` CLI flags; load spork source at boot
- **AGENTS.md**: Update default port documentation
- **README.md**: Update connection instructions
- **No impact** on VS Code or Emacs users — they use stdio subprocess, not TCP
- **Breaking** for any automated scripts that `nc` to port 9365 — change to 9364
