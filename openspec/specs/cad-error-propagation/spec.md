## ADDED Requirements

### Requirement: CAD operations validate numeric inputs

All CAD primitive operations (box, sphere, cylinder, cone, torus, rect, circle) SHALL validate that numeric dimensions are positive before attempting shape construction. Non-positive values SHALL result in a descriptive error returned to the caller rather than a process crash.

#### Scenario: Negative dimension returns error

- **WHEN** a CAD operation is called with a negative dimension (e.g., `(box -1)`, `(sphere -5)`)
- **THEN** the operation returns an error describing the invalid value
- **AND** the process continues running normally

#### Scenario: Zero dimension returns error

- **WHEN** a CAD operation is called with a zero dimension (e.g., `(box 0 10 10)`)
- **THEN** the operation returns an error describing the invalid value
- **AND** the process continues running normally

#### Scenario: Valid dimensions succeed

- **WHEN** a CAD operation is called with all-positive dimensions
- **THEN** the operation returns a valid shape as expected

### Requirement: Boolean operations surface empty results as errors

Boolean operations (cut, common, fuse) SHALL detect when the result is an empty or null shape and return a descriptive error instead of panicking.

#### Scenario: Cut non-intersecting shapes returns error

- **WHEN** `cut` is called with two shapes that do not intersect
- **THEN** the operation returns an error indicating no intersection
- **AND** the process continues running normally

#### Scenario: Common non-intersecting shapes returns error

- **WHEN** `common` is called with two shapes that do not intersect
- **THEN** the operation returns an error indicating no intersection
- **AND** the process continues running normally

#### Scenario: Fuse with empty result returns error

- **WHEN** `fuse` produces an empty result
- **THEN** the operation returns an error indicating empty result
- **AND** the process continues running normally

#### Scenario: Valid boolean operation succeeds

- **WHEN** `cut`, `common`, or `fuse` is called with intersecting shapes
- **THEN** the operation returns a valid boolean result shape

### Requirement: Wire operations validate radius and distance

Wire operations (fillet, chamfer, offset) SHALL validate that numeric parameters are positive and return errors for invalid values.

#### Scenario: Negative fillet radius returns error

- **WHEN** `wire-fillet` is called with a negative radius
- **THEN** the operation returns a descriptive error
- **AND** the process continues running normally

#### Scenario: Negative chamfer distance returns error

- **WHEN** `wire-chamfer` is called with a negative distance
- **THEN** the operation returns a descriptive error
- **AND** the process continues running normally

### Requirement: CAD errors propagate to the Janet REPL

Errors from CAD operations SHALL propagate through the C bridge via `janet_panic` and be caught by the REPL error-recovery mechanism, which SHALL return the error message to the client without terminating the connection.

#### Scenario: Invalid input error reaches client

- **WHEN** a client sends `(box -1)` over the TCP REPL
- **THEN** the client receives an error message describing the invalid input
- **AND** the REPL continues accepting further commands

#### Scenario: Boolean error reaches client

- **WHEN** a client sends `(cut a b)` where `a` and `b` do not intersect
- **THEN** the client receives an error message describing the empty result
- **AND** the REPL continues accepting further commands
