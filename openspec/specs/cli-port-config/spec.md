## ADDED Requirements

### Requirement: CLI `--port` flag
The system SHALL accept a `--port <number>` or `--port=<number>` CLI argument to override the netrepl default port.

#### Scenario: Port set via adjacent argument
- **WHEN** the user runs `rojcad --port 8000`
- **THEN** the netrepl server SHALL listen on port 8000

#### Scenario: Port set via equals syntax
- **WHEN** the user runs `rojcad --port=8000`
- **THEN** the netrepl server SHALL listen on port 8000

#### Scenario: Port defaults to 9365
- **WHEN** the user runs `rojcad` without a `--port` flag
- **THEN** the netrepl server SHALL listen on port 9365

#### Scenario: Invalid port is rejected
- **WHEN** the user runs `rojcad --port 99999`
- **THEN** the program SHALL print an error and exit with a non-zero status

### Requirement: Port is a Janet dynamic variable
The system SHALL expose the port value to the Janet VM as the `*netrepl-port*` dynamic variable before boot code executes.

#### Scenario: boot.janet reads the dynamic variable
- **WHEN** boot.janet runs and `--port 8000` was specified
- **THEN** `(dyn '*netrepl-port*')` SHALL return 8000

#### Scenario: Dynamic variable absent means default
- **WHEN** boot.janet runs without `--port`
- **THEN** `(dyn '*netrepl-port*')` SHALL return nil, and the server SHALL fall back to 9365
