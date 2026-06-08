## Context

The Janet netrepl server currently hardcodes port 9000 in `boot.janet`. This port overlaps with common development tools (e.g., PHP, web proxies). The IANA-registered port for Janet is 9365.

## Goals / Non-Goals

**Goals:**
- Change the default netrepl port from 9000 to 9365
- Update any documentation referencing the old port

**Non-Goals:**
- Adding command-line or config-file port overrides (could be a future change)
- Changing the REPL protocol or behavior

## Decisions

- **Single source of truth**: The port is defined once in `boot.janet` as `(def port 9365)`. No other files need a runtime port value — only a doc comment in `src/main.rs` references 9000.
- **No env-var/config override**: Not needed for this change. A `--port` flag could be added later if desired.

## Risks / Trade-offs

- **Breaking change**: Existing scripts/tools connecting to port 9000 will break. Mitigation: clearly document the new port in the startup message and release notes.
- **Minimal blast radius**: Only two files change (`boot.janet`, `src/main.rs`), both in the same repo. No external integrations depend on port 9000.
