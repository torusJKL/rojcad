## ADDED Requirements

### Requirement: Revolve Face to Solid
The system SHALL provide a `revolve` function that creates a Solid from a Face by revolving it about an axis.

`(revolve face :a angle)` revolves about the Z axis through the origin by the given angle in degrees. Returns ShapeData (SOLID).

Keywords: `:a` (angle in degrees), `:ar` (angle in radians), `:c` (axis origin point, default `[0 0 0]`), `:dir` (axis direction, default `[0 0 1]`), `:eager`, `:hide`.

#### Scenario: Revolve about Z axis by default
- **WHEN** user calls `(revolve (rect :w 10 :h 20) :a 360)`
- **THEN** the system returns a ShapeData with shape type SOLID

#### Scenario: Revolve partial angle
- **WHEN** user calls `(revolve (rect :w 10 :h 20) :a 180)`
- **THEN** the system returns a ShapeData with shape type SOLID representing a half-revolution

#### Scenario: Revolve about custom axis
- **WHEN** user calls `(revolve face :a 180 :c [0 0 0] :dir [0 1 0])`
- **THEN** the system returns a ShapeData with shape type SOLID revolved about the Y axis

#### Scenario: Revolve with radians
- **WHEN** user calls `(revolve face :ar 3.14159)`
- **THEN** the system returns a ShapeData with shape type SOLID revolved by π radians (180 degrees)

#### Scenario: Revolve errors on non-face input
- **WHEN** user calls `(revolve (box 10) :a 360)`
- **THEN** the system signals an error
