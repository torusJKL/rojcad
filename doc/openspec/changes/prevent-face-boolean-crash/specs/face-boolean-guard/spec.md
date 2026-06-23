## ADDED Requirements

### Requirement: Guard rejects Face-Face cut

The system SHALL reject `cut` when both input shapes are `Face` type, returning an error message with the operation name, the cause (two planar faces), and the workaround (extrude to solids). The `common` and `fuse` operations are NOT guarded — OCCT investigation confirmed they handle Face-Face input correctly.

#### Scenario: cut with two faces returns error
- **WHEN** `cut` is called with two shapes where both `shape_type()` are `Face`
- **THEN** the function returns `Err` containing the string `"cannot cut two planar faces"`

#### Scenario: cut with Face and Solid passes through
- **WHEN** `cut` is called with a `Face` and a `Solid`
- **THEN** the operation proceeds normally (no guard error)

#### Scenario: cut with two Solids passes through
- **WHEN** `cut` is called with two `Solid` shapes
- **THEN** the operation proceeds normally (no guard error)

### Requirement: Error message includes workaround guidance

The error message SHALL tell the user what to do to fix the problem (extrude faces to solids).

#### Scenario: error message contains extrusion hint
- **WHEN** `cut` is called with two `Face` shapes and returns `Err`
- **THEN** the error string contains `"Extrude"` or `"prism"`
