## ADDED Requirements

### Requirement: I/O functions preserve behavior

#### Scenario: write-step succeeds
- **WHEN** `(write-step shape "/tmp/test.step")` is called
- **THEN** a STEP file is written and nil is returned
- **WHEN** the path is invalid
- **THEN** an error is signaled

#### Scenario: read-step loads a STEP file
- **WHEN** `(read-step "/tmp/test.step")` is called
- **THEN** it returns a rojcad/shape abstract value
