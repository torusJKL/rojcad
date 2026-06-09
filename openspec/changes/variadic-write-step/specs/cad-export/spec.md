## MODIFIED Requirements

### Requirement: Export shape to STEP

The system SHALL provide a function `write-step` that exports zero or more shapes to the STEP file format using OCCT's STEPControl_Writer.

The function SHALL accept a string file path as the first positional argument, followed by zero or more `rojcad/shape` abstract values as variadic positional arguments:
`(write-step path & shapes)`

When called with only a path (no shape arguments), the function SHALL export all currently visible shapes registered in the viewer registry.

When called with a single shape, the function SHALL produce a STEP file identical to the previous single-shape behavior.

When called with multiple shapes, the function SHALL transfer each shape to the same STEP writer before writing the file, producing a multi-root-entity STEP file.

The function SHALL return nil on success.

When called with no shape arguments and no visible shapes are registered, the system SHALL signal a Janet error with the message "no visible shapes to export".

If any shape cannot be transferred to the STEP writer, or the file cannot be written, the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Export all visible shapes
- **WHEN** user calls `(show (make-box 10 20 30))` then `(show (make-sphere 10))` then `(write-step "visible.step")`
- **THEN** the file `visible.step` exists and contains both the box and sphere

#### Scenario: Export single shape
- **WHEN** user calls `(write-step "box.step" (make-box 10 20 30))`
- **THEN** the file `box.step` exists and is a valid STEP file containing the box

#### Scenario: Export multiple shapes explicitly
- **WHEN** user calls `(write-step "assembly.step" (make-box 10 20 30) (make-sphere 10))`
- **THEN** the file `assembly.step` exists and is a valid STEP file containing both the box and the sphere as separate root entities

#### Scenario: No visible shapes errors
- **WHEN** user calls `(write-step "empty.step")` with no visible shapes
- **THEN** the system SHALL signal a Janet error "no visible shapes to export"

#### Scenario: Invalid path with shapes
- **WHEN** user calls `(write-step "/nonexistent/output.step" (make-box 10 10 10))`
- **THEN** the system SHALL signal a Janet error
