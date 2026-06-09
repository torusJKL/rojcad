## ADDED Requirements

### Requirement: Spork netrepl server runs on port 9365

The system SHALL run a spork/netrepl protocol server on TCP port 9365 alongside the raw REPL server.

#### Scenario: Spork server starts on default port

- **WHEN** the system boots without explicit port overrides
- **THEN** a spork netrepl server SHALL be listening on TCP port 9365

#### Scenario: Spork server accepts Conjure client connections

- **WHEN** a Conjure (Neovim) client connects to port 9365
- **THEN** the server SHALL complete the spork protocol handshake (receive client name, send prompt, accept input)
- **AND** the client SHALL be able to evaluate Janet expressions

#### Scenario: Spork server accepts spork CLI client connections

- **WHEN** a user connects via `janet -e "(import spork/netrepl) (netrepl/client)"`
- **THEN** the server SHALL accept the connection and provide an interactive REPL session

### Requirement: Spork server shares the same core environment

The spork netrepl server SHALL evaluate expressions in the same `core-env` as the raw REPL server, providing access to all CAD functions and Janet built-ins.

#### Scenario: CAD functions available via spork REPL

- **WHEN** a spork client evaluates a CAD function (e.g., `(box 10 20 30)`)
- **THEN** the result SHALL be the same as evaluating it via the raw REPL

### Requirement: Spork port is configurable

The spork netrepl server port SHALL be configurable via a `--spork-port` CLI flag.

#### Scenario: Spork port set via --spork-port

- **WHEN** rojcad is started with `--spork-port 9367`
- **THEN** the spork netrepl server SHALL listen on port 9367

#### Scenario: Spork port set via --spork-port=PORT syntax

- **WHEN** rojcad is started with `--spork-port=9367`
- **THEN** the spork netrepl server SHALL listen on port 9367

### Requirement: Both servers can run concurrently

The raw REPL and spork REPL servers SHALL run concurrently without interfering with each other.

#### Scenario: Both ports accept connections simultaneously

- **WHEN** a client is connected to the raw REPL on port 9364
- **AND** another client connects to the spork REPL on port 9365
- **THEN** both connections SHALL be handled independently

#### Scenario: Spork server failure does not affect raw REPL

- **WHEN** the spork server fails to bind (e.g., port in use)
- **THEN** the raw REPL SHALL continue to operate normally
- **AND** an error message SHALL be printed to stderr

### Requirement: Startup banner reports both ports

The system SHALL print a startup banner showing both the raw REPL port and the spork REPL port.

#### Scenario: Banner on default ports

- **WHEN** rojcad starts with default ports
- **THEN** the banner SHALL include both port 9364 (raw) and 9365 (spork)

#### Scenario: Banner with custom ports

- **WHEN** rojcad starts with `--raw-port 9000 --spork-port 9001`
- **THEN** the banner SHALL show port 9000 for raw and 9001 for spork
