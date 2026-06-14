## MODIFIED Requirements

### Requirement: Export shape to STL

The system SHALL provide a function `write-stl` that exports one or more shapes' triangulated meshes to the STL file format using OCCT's StlAPI_Writer.

The function SHALL accept a string file path as the first positional argument, followed by zero or more `rojcad/shape` abstract values as variadic positional arguments:
`(write-stl path & shapes)`

When called with only a path (no shape arguments), the function SHALL export all currently visible shapes registered in the viewer registry. All shapes SHALL be combined into a single OCCT compound and written as one STL file.

When called with a single shape, the function SHALL produce an STL file identical to the previous single-shape behavior.

When called with multiple shapes, the function SHALL combine all shapes into a single OCCT compound using `Compound::from_shapes` before writing via `StlAPI_Writer`.

The function SHALL use a default triangulation tolerance of 0.001 for mesh generation.

The function SHALL return nil on success.

When called with no shape arguments and no visible shapes are registered, the system SHALL signal a Janet error with the message "no visible shapes to export".

If the file cannot be written or a shape cannot be triangulated, the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Export all visible shapes to STL
- **WHEN** user calls `(show (make-box 10 20 30))` then `(show (make-sphere 10))` then `(write-stl "visible.stl")`
- **THEN** the file `visible.stl` exists and contains the meshes of both the box and sphere as a single compound

#### Scenario: Export single shape to STL
- **WHEN** user calls `(write-stl "box.stl" (make-box 10 20 30))`
- **THEN** the file `box.stl` exists and is a valid STL file containing the box mesh

#### Scenario: Export multiple shapes explicitly to STL
- **WHEN** user calls `(write-stl "assembly.stl" (make-box 10 20 30) (make-sphere 10))`
- **THEN** the file `assembly.stl` exists and is a valid STL file containing both meshes

#### Scenario: No visible shapes errors on STL export
- **WHEN** user calls `(write-stl "empty.stl")` with no visible shapes
- **THEN** the system SHALL signal a Janet error "no visible shapes to export"

#### Scenario: Invalid path with STL export
- **WHEN** user calls `(write-stl "/nonexistent/output.stl" (make-box 10 10 10))`
- **THEN** the system SHALL signal a Janet error

#### Scenario: Invalid path with no shapes errors
- **WHEN** user calls `(write-stl 123)` (path type error)
- **THEN** the system SHALL signal a Janet type error

