## ADDED Requirements

### Requirement: TCP REPL server

The system SHALL listen on TCP port 9000 on loopback (127.0.0.1) and provide a Janet REPL to connected clients.

The REPL server SHALL be implemented in Janet (in `boot.janet`) using `net/listen`, `ev/spawn`, and the built-in `repl` function on client streams.

Each connected client SHALL receive their own REPL session running in a separate Janet fiber (green thread) via `ev/spawn`.

The REPL SHALL have access to all registered CAD functions (make-box, make-sphere, cut, common, shape-type, hide, show, visible?, write-step, write-stl).

The system SHALL print a banner to stderr on startup: `"◆ rojcad ready — connect via: nc 127.0.0.1 9000"`.

The system SHALL print `"● client connected"` / `"● client disconnected"` to stderr for each connection.

If the port is in use, the system SHALL signal a Janet error and exit with a non-zero status code.

#### Scenario: Connect with nc
- **WHEN** a user runs `nc 127.0.0.1 9000`
- **THEN** they receive a Janet REPL prompt and can evaluate expressions

#### Scenario: Create shape via REPL
- **WHEN** a user connects via TCP and types `(make-box 10 20 30)`
- **THEN** the REPL responds with `#<Shape(SOLID)>`

#### Scenario: Multiple concurrent clients
- **WHEN** two users connect simultaneously via separate nc instances
- **THEN** both receive independent REPL sessions that do not interfere

#### Scenario: Port conflict
- **WHEN** the port 9000 is already in use at startup
- **THEN** the system prints an error and exits with a non-zero status code
