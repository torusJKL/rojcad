## ADDED Requirements

### Requirement: Create a sketch
The system SHALL provide a `sketch` function that creates a new sketch on a workplane.

Keywords: `:plane` (workplane, default `:xy`), `:at` (position offset, default `[0 0 0]`).

Returns a `rojcad/sketch` abstract value (not a shape).

#### Scenario: Default sketch is on XY plane at origin
- **WHEN** user calls `(sketch)` or `(sketch :plane :xy)`
- **THEN** the system returns a sketch on the XY plane with cursor at (0, 0)

#### Scenario: Sketch on custom workplane
- **WHEN** user calls `(sketch :plane :xz :at [10 0 5])`
- **THEN** the system returns a sketch on the XZ plane offset by [10, 0, 5]

### Requirement: Move cursor without drawing
The system SHALL provide a `move-to` function that lifts the pen and positions the cursor without adding an edge.

Takes positional arguments `(move-to sketch x y)` in workplane coordinates. Returns a new sketch.

#### Scenario: Move-to repositions cursor
- **WHEN** user calls `(move-to (sketch) 5 5)`
- **THEN** the system returns a new sketch with cursor at (5, 5) and no edges added

### Requirement: Draw line to absolute position
The system SHALL provide a `line-to` function that draws a straight edge from the current cursor to an absolute (x, y) position on the workplane.

Takes positional arguments `(line-to sketch x y)`. Returns a new sketch.

#### Scenario: Line-to adds one edge
- **WHEN** user calls `(line-to (sketch) 10 0)`
- **THEN** the system returns a new sketch with cursor at (10, 0) and one edge in the edge list

### Requirement: Draw line by relative offset
The system SHALL provide `line-dx`, `line-dy`, and `line-dx-dy` functions that draw edges by relative offsets from the current cursor position.

`(line-dx sketch dx)`, `(line-dy sketch dy)`, `(line-dx-dy sketch dx dy)`. Each returns a new sketch.

#### Scenario: Line-dx draws horizontal line
- **WHEN** user calls `(-> (sketch) (line-to 10 0) (line-dx 5))`
- **THEN** the system returns a sketch with cursor at (15, 0) and two edges

### Requirement: Draw arc
The system SHALL provide an `arc-to` function that draws a circular arc through three points: current cursor, intermediate point (x2, y2), and end point (x3, y3).

`(arc-to sketch x2 y2 x3 y3)`. Returns a new sketch.

#### Scenario: Arc-to adds one curved edge
- **WHEN** user calls `(-> (sketch) (line-to 10 0) (arc-to 10 10 0 10))`
- **THEN** the system returns a sketch with cursor at (0, 10) and two edges (one straight, one arc)

### Requirement: Close sketch as Face
The system SHALL provide a `close-sketch` function that closes the sketch back to the first point and returns a ShapeData wrapping a Face.

If the cursor is not at the first point, a closing edge is added automatically. Returns ShapeData (FACE).

#### Scenario: Close-sketch returns a Face
- **WHEN** user calls `(-> (sketch) (line-to 10 0) (line-to 10 10) (line-to 0 10) (close-sketch))`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Close-sketch adds closing edge when not at start
- **WHEN** user calls `(-> (sketch) (line-to 10 0) (close-sketch))`
- **THEN** the system returns a ShapeData with shape type FACE containing a closed rectangle

### Requirement: Build unclosed Wire
The system SHALL provide a `build-wire` function that returns a ShapeData wrapping a Wire without closing the sketch.

Returns ShapeData (WIRE).

#### Scenario: Build-wire returns a Wire without closing
- **WHEN** user calls `(-> (sketch) (line-to 10 0) (line-to 10 10) (build-wire))`
- **THEN** the system returns a ShapeData with shape type WIRE and no closing edge

### Requirement: Empty sketch closes as empty face
- **WHEN** user calls `(-> (sketch) (close-sketch))`
- **THEN** the system signals an error (no edges to close)
