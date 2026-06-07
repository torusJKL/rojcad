## ADDED Requirements

### Requirement: Create rectangle
The system SHALL provide a `rect` function that creates a rectangular Face on a workplane.

Keywords: `:w` / `:d` (dimensions), `:wire` (return Wire instead of Face), `:plane` (workplane, default `:xy`), `:at` (position offset, default `[0 0 0]`), `:eager`, `:hide`.

#### Scenario: Rectangle creates a Face by default
- **WHEN** user calls `(rect :w 10 :h 20)` or `(rect 10 20)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Rectangle with :wire keyword returns a Wire
- **WHEN** user calls `(rect :w 10 :h 20 :wire)`
- **THEN** the system returns a ShapeData with shape type WIRE

#### Scenario: Rectangle on custom workplane
- **WHEN** user calls `(rect :w 10 :h 20 :plane :xz :at [5 0 0])`
- **THEN** the system returns a rectangle on the XZ plane offset by [5, 0, 0]

### Requirement: Create circle
The system SHALL provide a `circle` function that creates a circular Face on a workplane.

Keywords: `:r` (radius), `:wire`, `:plane`, `:at`, `:eager`, `:hide`.

#### Scenario: Circle creates a Face by default
- **WHEN** user calls `(circle :r 5)` or `(circle 5)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Circle with :wire keyword returns a Wire
- **WHEN** user calls `(circle :r 5 :wire)`
- **THEN** the system returns a ShapeData with shape type WIRE

### Requirement: Create polygon
The system SHALL provide a `polygon` function that creates a polygonal Face from a list of 2D points.

Keywords: `:pts` (array of [x y] tuples), `:wire`, `:plane`, `:at`, `:eager`, `:hide`.

#### Scenario: Polygon from point list creates a Face
- **WHEN** user calls `(polygon :pts [[0 0] [10 0] [10 10] [0 10]])`
- **THEN** the system returns a ShapeData with shape type FACE constructed from the given points

#### Scenario: Polygon validates minimum points
- **WHEN** user calls `(polygon :pts [[0 0]])` with fewer than 3 points
- **THEN** the system signals an error

### Requirement: Helper queries
The system SHALL provide `wire?`, `face?`, and `solid?` predicates that inspect a shape's type.

#### Scenario: wire? returns true for Wire shapes
- **WHEN** user calls `(wire? (rect :w 10 :h 20 :wire))`
- **THEN** the system returns true

#### Scenario: face? returns true for Face shapes  
- **WHEN** user calls `(face? (rect :w 10 :h 20))`
- **THEN** the system returns true

#### Scenario: solid? returns true for Solid shapes
- **WHEN** user calls `(solid? (box 10))`
- **THEN** the system returns true
