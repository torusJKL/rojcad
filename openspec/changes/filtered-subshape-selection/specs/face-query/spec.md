## ADDED Requirements

### Requirement: Face metadata query

The system SHALL provide a Janet function `face-info` that returns an array of structs, one per face of the input shape. Each struct SHALL contain metadata about a single face.

`(face-info shape)` takes a ShapeData and returns an array of structs.

Each struct SHALL contain these keys:
- `:index` (integer, 1-based) — stable index for use with `get-face`
- `:type` (string) — surface type: `"plane"`, `"cylinder"`, `"sphere"`, `"cone"`, `"torus"`, `"bezier-surface"`, `"bspline-surface"`, `"surface-of-revolution"`, `"surface-of-extrusion"`, `"offset-surface"`, `"other"`
- `:area` (number) — surface area of the face
- `:center` (tuple [x y z]) — center of mass
- `:normal` (tuple [x y z]) — normal vector at center; `(0 0 0)` for non-planar faces
- `:axis` (tuple [x y z]) — axis direction for cylindrical/conical/toroidal faces; `(0 0 0)` for planar/spherical/other
- `:radius` (number) — radius for cylindrical/spherical faces; `0.0` for planar faces

#### Scenario: Query faces of a box
- **WHEN** user calls `(face-info (box 10 20 30))`
- **THEN** the system returns an array of 6 structs
- **AND** each struct has `:type` equal to `"plane"`
- **AND** each struct has `:area` equal to `200.0`, `300.0`, or `600.0` (matching the box faces)
- **AND** each struct has a non-zero `:normal` vector
- **AND** each struct has `:axis` equal to `(0 0 0)` (planes have no axis)
- **AND** each struct has `:radius` equal to `0.0`

#### Scenario: Query faces of a cylinder
- **WHEN** user calls `(face-info (cylinder :r 5 :h 10))`
- **THEN** the system returns an array of 3 structs
- **AND** exactly one struct has `:type` equal to `"cylinder"` with `:radius` equal to `5.0`
- **AND** exactly two structs have `:type` equal to `"plane"`

#### Scenario: Query faces of a sphere
- **WHEN** user calls `(face-info (sphere :r 5))`
- **THEN** the system returns an array of 1 struct
- **AND** the struct has `:type` equal to `"sphere"` with `:radius` equal to `5.0`

#### Scenario: Error on null shape
- **WHEN** user calls `(face-info nil)`
- **THEN** the system signals an error

#### Scenario: Iteration over results
- **WHEN** user calls `(each f (face-info (box 10 20 30)) (print (f :type)))`
- **THEN** the system prints `"plane"` six times

#### Scenario: Filter faces by predicate
- **WHEN** user calls `(filter (fn [f] (= (f :type) "cylinder")) (face-info shape))`
- **THEN** the system returns only structs with `:type` equal to `"cylinder"`
