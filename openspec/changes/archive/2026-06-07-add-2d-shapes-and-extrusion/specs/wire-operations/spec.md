## ADDED Requirements

### Requirement: Wire to Face conversion
The system SHALL provide a `wire-to-face` function that converts a Wire-shape to a Face by filling its boundary.

`(wire-to-face wire)` takes a ShapeData with shape type WIRE and returns a ShapeData with shape type FACE.

#### Scenario: wire-to-face converts a Wire to a Face
- **WHEN** user calls `(wire-to-face (rect :w 10 :h 20 :wire))`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: wire-to-face errors on non-wire input
- **WHEN** user calls `(wire-to-face (box 10))`
- **THEN** the system signals an error

### Requirement: Wire fillet
The system SHALL provide a `wire-fillet` function that rounds all vertices of a closed wire by a given radius.

`(wire-fillet wire :r radius)` returns a new ShapeData with shape type WIRE.

#### Scenario: wire-fillet rounds corners
- **WHEN** user calls `(wire-fillet (rect :w 10 :h 20 :wire) :r 2)`
- **THEN** the system returns a ShapeData with shape type WIRE whose vertices are rounded

#### Scenario: wire-fillet errors on non-wire input
- **WHEN** user calls `(wire-fillet (box 10) :r 2)`
- **THEN** the system signals an error

### Requirement: Wire chamfer
The system SHALL provide a `wire-chamfer` function that bevels all vertices of a closed wire by a given distance.

`(wire-chamfer wire :d distance)` returns a new ShapeData with shape type WIRE.

#### Scenario: wire-chamfer bevels corners
- **WHEN** user calls `(wire-chamfer (rect :w 10 :h 20 :wire) :d 2)`
- **THEN** the system returns a ShapeData with shape type WIRE whose vertices are chamfered

### Requirement: Wire offset
The system SHALL provide a `wire-offset` function that creates a parallel offset of a closed wire.

`(wire-offset wire :d distance)` returns a new ShapeData with shape type WIRE.

#### Scenario: wire-offset creates parallel offset
- **WHEN** user calls `(wire-offset (rect :w 10 :h 20 :wire) :d 2)`
- **THEN** the system returns a ShapeData with shape type WIRE that is 2 units larger in all directions
