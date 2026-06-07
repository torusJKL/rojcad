## ADDED Requirements

### Requirement: Default port is 9365
The Janet netrepl server SHALL default to port 9365 when no port is explicitly configured.

#### Scenario: Server starts on default port
- **WHEN** the server starts without an explicit port override
- **THEN** the server SHALL bind to TCP port 9365

#### Scenario: Server logs the port
- **WHEN** the server starts successfully
- **THEN** the startup message SHALL display port 9365
