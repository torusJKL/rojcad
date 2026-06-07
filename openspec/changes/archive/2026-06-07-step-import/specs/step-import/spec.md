## ADDED Requirements

### Requirement: Read STEP file from disk

The system SHALL provide a `(read-step path)` Janet function that loads a STEP file from disk and returns a rojcad shape value. The shape SHALL support all standard operations (`show`, `hide`, `shape-type`, boolean ops, export).

#### Scenario: Read valid STEP file successfully
- **WHEN** `(read-step "/tmp/model.step")` is called and the file is a valid STEP file
- **THEN** a shape value is returned with a valid shape_id
- **AND** calling `(show shape)` displays it in the viewer

#### Scenario: File not found signals error
- **WHEN** `(read-step "/nonexistent.step")` is called
- **THEN** an error is signaled with a message containing "not found" or the OS-level error

#### Scenario: Invalid STEP file signals error
- **WHEN** `(read-step "/tmp/not-a-step.bin")` is called with a file that is not valid STEP
- **THEN** an error is signaled with a descriptive message

#### Scenario: Round-trip write then read STEP
- **WHEN** a box is created, written to STEP via `write-step`, and read back via `read-step`
- **THEN** the re-imported shape has a valid shape_type (e.g., SOLID or COMPOUND)
