## ADDED Requirements

### Requirement: Extrude Face to Solid
The system SHALL provide an `extrude` function that creates a Solid from a Face by extruding along a direction.

`(extrude face :h height)` extrudes along the face's normal vector by the given height. Returns ShapeData (SOLID).

Keywords: `:h` (height, required), `:z`/`:x`/`:y` (cardinal axis override), `:dir [dx dy dz]` (custom direction), `:both` (extrude both sides), `:eager`, `:hide`.

#### Scenario: Extrude along face normal
- **WHEN** user calls `(extrude (rect :w 10 :h 20) :h 5)`
- **THEN** the system returns a ShapeData with shape type SOLID

#### Scenario: Extrude along global Z axis
- **WHEN** user calls `(extrude (rect :w 10 :h 20) :h 5 :z)`
- **THEN** the system returns a ShapeData with shape type SOLID extruded along the global Z axis

#### Scenario: Extrude along custom direction
- **WHEN** user calls `(extrude (rect :w 10 :h 20) :h 5 :dir [0 0 -1])`
- **THEN** the system returns a ShapeData with shape type SOLID extruded in the negative Z direction

#### Scenario: Extrude both sides
- **WHEN** user calls `(extrude (rect :w 10 :h 20) :h 5 :both)`
- **THEN** the system returns a ShapeData with shape type SOLID that extends 5 units on each side of the original face

#### Scenario: Extrude errors on non-face input
- **WHEN** user calls `(extrude (box 10) :h 5)`
- **THEN** the system signals an error

### Requirement: One-shot polygon extrusion
The system SHALL provide an `extrude-polygon` function that creates a Solid directly from a list of 2D points and a height.

`(extrude-polygon pts :h height)` or `(extrude-polygon pts height)` where pts is an array of [x y] tuples.

Keywords: `:h` (height), `:plane`, `:at`, `:eager`, `:hide`.

#### Scenario: extrude-polygon creates Solid from points
- **WHEN** user calls `(extrude-polygon [[0 0] [10 0] [10 10] [0 10]] :h 20)`
- **THEN** the system returns a ShapeData with shape type SOLID

#### Scenario: extrude-polygon with positional height
- **WHEN** user calls `(extrude-polygon [[0 0] [10 0] [10 10] [0 10]] 20)`
- **THEN** the system returns a ShapeData with shape type SOLID
