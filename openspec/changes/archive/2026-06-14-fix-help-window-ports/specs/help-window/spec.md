## MODIFIED Requirements

### Requirement: Help window displays REPL connection info

**Previous version:**
The help window SHALL display how to connect to the REPL via netcat, showing `nc 127.0.0.1 9365`.

The help window SHALL display how to connect to the raw TCP REPL via netcat.

#### Scenario: Connection info shown
- **WHEN** the help window is visible
- **THEN** it shows: nc 127.0.0.1 9364

### Requirement: Help window displays CLI arguments

**Previous version:**
The help window SHALL display available command-line arguments, which were listed as `--headless, --port, --eval`.

The help window SHALL display available command-line arguments.

#### Scenario: CLI args shown
- **WHEN** the help window is visible
- **THEN** it lists: --headless, --raw-port, --spork-port, --eval
