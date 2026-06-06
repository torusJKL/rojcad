## ADDED Requirements

### Requirement: Export shape to STEP

The system SHALL provide a function `write-step` that exports a shape to the STEP file format using OCCT's STEPControl_Writer.

The function SHALL accept two required positional arguments: a `rojcad/shape` abstract value and a string file path.

The function SHALL return nil on success.

If the file cannot be written, the OCCT transfer fails, or the path is invalid, the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Export box to STEP
- **WHEN** user calls `(write-step (make-box 10 20 30) "box.step")`
- **THEN** the file `box.step` exists and is a valid STEP file containing the box

#### Scenario: Invalid path
- **WHEN** user calls `(write-step (make-box 10 10 10) "/nonexistent/output.step")`
- **THEN** the system SHALL signal a Janet error

### Requirement: Export shape to STL

The system SHALL provide a function `write-stl` that exports a shape's triangulated mesh to the STL file format using OCCT's StlAPI_Writer.

The function SHALL accept two required positional arguments: a `rojcad/shape` abstract value and a string file path.

The function SHALL use a default triangulation tolerance (0.001) for mesh generation.

The function SHALL return nil on success.

If the file cannot be written or the shape cannot be triangulated, the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Export sphere to STL
- **WHEN** user calls `(write-stl (make-sphere 10) "sphere.stl")`
- **THEN** the file `sphere.stl` exists and is a valid STL file containing the sphere mesh
