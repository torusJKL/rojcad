## ADDED Requirements

### Requirement: Face offset

The system SHALL provide a Janet function `face-offset` that creates a parallel offset of a face by a given distance.

`(face-offset face :d distance)` takes a Face-shaped ShapeData. It SHALL return a new ShapeData with shape type FACE representing the offset surface.

The `:d` keyword SHALL be required. The `:eager` and `:hide` keywords SHALL be supported following the same pattern as other operations.

#### Scenario: Offset a planar face
- **WHEN** user calls `(face-offset (get-face (box 10 20 30) 1) :d 2)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Face offset errors on non-face input
- **WHEN** user calls `(face-offset (box 10 20 30) :d 2)`
- **THEN** the system signals an error

#### Scenario: Face offset with eager/hide
- **WHEN** user calls `(face-offset face :d 2 :eager :hide)`
- **THEN** the system returns a ShapeData that is tessellated and hidden

### Requirement: Face fillet

The system SHALL provide a Janet function `face-fillet` that rounds all vertices of a face's boundary edges by a given radius (2D fillet).

`(face-fillet face :r radius)` takes a Face-shaped ShapeData. It SHALL return a new ShapeData with shape type FACE whose boundary vertices are rounded.

The `:r` keyword SHALL be required. The `:eager` and `:hide` keywords SHALL be supported.

#### Scenario: Face fillet rounds corners
- **WHEN** user calls `(def f (rect :w 10 :h 20)) (face-fillet f :r 2)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Face fillet errors on non-face input
- **WHEN** user calls `(face-fillet (box 10 20 30) :r 2)`
- **THEN** the system signals an error

### Requirement: Face chamfer

The system SHALL provide a Janet function `face-chamfer` that bevels all vertices of a face's boundary edges by a given distance (2D chamfer).

`(face-chamfer face :d distance)` takes a Face-shaped ShapeData. It SHALL return a new ShapeData with shape type FACE whose boundary vertices are chamfered.

The `:d` keyword SHALL be required. The `:eager` and `:hide` keywords SHALL be supported.

#### Scenario: Face chamfer bevels corners
- **WHEN** user calls `(def f (rect :w 10 :h 20)) (face-chamfer f :d 2)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Face chamfer errors on non-face input
- **WHEN** user calls `(face-chamfer (box 10 20 30) :d 2)`
- **THEN** the system signals an error

### Requirement: Face to workplane

The system SHALL provide a Janet function `face-workplane` that creates a workplane aligned to a face.

`(face-workplane face)` takes a Face-shaped ShapeData. It SHALL return a workplane value that can be passed to `sketch` for 2D drawing on the face's plane.

The workplane SHALL be positioned at the face's center of mass with Z-axis aligned to the face normal.

#### Scenario: Workplane from a planar face
- **WHEN** user calls `(face-workplane (get-face (box 10 20 30) 1))`
- **THEN** the system returns a workplane value

#### Scenario: Sketch on extracted face workplane
- **WHEN** user calls the following:
  ```janet
  (def wp (face-workplane (get-face (box 10 20 30) 1)))
  (def s (sketch wp
            (move-to 2 2)
            (line-to 8 2)
            (line-to 8 8)
            (line-to 2 8)
            (close-sketch)))
  ```
- **THEN** the system returns a valid ShapeData

#### Scenario: Face workplane errors on non-face input
- **WHEN** user calls `(face-workplane (box 10 20 30))`
- **THEN** the system signals an error
