## ADDED Requirements

### Requirement: REPL catches evaluation errors

The TCP REPL SHALL catch all evaluation errors and return them to the client as error strings, allowing the REPL loop to continue processing subsequent commands.

#### Scenario: Arity mismatch error is caught
- **WHEN** client sends an expression that signals an error (e.g., `(show a b)` where `show` expects 1 argument)
- **THEN** the REPL returns the error string to the client
- **AND** the REPL continues accepting further commands

#### Scenario: Undefined symbol error is caught
- **WHEN** client sends an expression referencing an undefined symbol (e.g., `(nonexistent-function 42)`)
- **THEN** the REPL returns the error string to the client
- **AND** the REPL continues accepting further commands

#### Scenario: Compilation error is caught
- **WHEN** client sends a malformed expression that fails compilation
- **THEN** the REPL returns the compile error string to the client
- **AND** the REPL continues accepting further commands

### Requirement: `try-catch` catches errors from any body

The `try-catch` function SHALL catch errors from its body argument using an error-protected fiber, rather than propagating errors to the caller.

#### Scenario: Error in body is caught by err-handler
- **WHEN** `try-catch` is called with a body that signals an error
- **THEN** the err-handler function is called with the error value
- **AND** the caller of `try-catch` receives the err-handler's return value

#### Scenario: Successful body returns normally
- **WHEN** `try-catch` is called with a body that completes without error
- **THEN** the err-handler is NOT called
- **AND** the caller receives the body's return value

### Requirement: `ev/go` applies error masking to REPL fibers

When the REPL accepts a new connection, the connection-handler fiber SHALL be created with error masking automatically applied by `ev/go`.

#### Scenario: ev/go creates fiber with error masking
- **WHEN** a new client connects
- **THEN** the connection-handler is scheduled via `ev/go` with a function argument (not a pre-built fiber)
- **AND** `ev/go` applies error masking flags to the resulting fiber

### Requirement: REPL closes connection on read error or EOF

When `net/read` returns nil or signals an error, the REPL SHALL cleanly close the connection.

#### Scenario: EOF closes connection
- **WHEN** `net/read` returns nil on the client stream
- **THEN** the REPL exits the read loop
- **AND** the TCP stream is closed

#### Scenario: Read error returns error string
- **WHEN** `net/read` signals an error on the client stream
- **THEN** the REPL returns the error string to the client
- **AND** the REPL exits the read loop
- **AND** the TCP stream is closed
