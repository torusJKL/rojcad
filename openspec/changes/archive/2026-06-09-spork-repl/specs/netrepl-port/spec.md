## MODIFIED Requirements

### Requirement: Default raw REPL port is 9364

The raw TCP REPL server SHALL default to port 9364 when no port is explicitly configured. The spork netrepl server defaults to 9365.

**Previous version:**
The Janet netrepl server SHALL default to port 9365 when no port is explicitly configured.

#### Scenario: Raw server starts on default port

- **WHEN** the raw REPL server starts without an explicit port override
- **THEN** the server SHALL bind to TCP port 9364

#### Scenario: Startup message shows raw port

- **WHEN** the system starts successfully
- **THEN** the startup message SHALL display both the raw REPL port (9364 by default) and the spork REPL port (9365 by default)

## ADDED Requirements

### Requirement: Raw REPL port is configurable via --raw-port

The raw TCP REPL server port SHALL be configurable via a `--raw-port` CLI flag.

#### Scenario: Raw port set via --raw-port

- **WHEN** rojcad is started with `--raw-port 9000`
- **THEN** the raw REPL server SHALL listen on port 9000

#### Scenario: Raw port set via --raw-port=PORT syntax

- **WHEN** rojcad is started with `--raw-port=9000`
- **THEN** the raw REPL server SHALL listen on port 9000

### Requirement: Raw REPL port global variable

The raw REPL port SHALL be injected into boot.janet as a `*raw-repl-port*` global definition by the Rust bootstrap, ensuring it's available before any Janet code runs.

#### Scenario: *raw-repl-port* is defined at boot

- **WHEN** rojcad starts with `--raw-port 9000`
- **THEN** `*raw-repl-port*` SHALL be 9000 within boot.janet
