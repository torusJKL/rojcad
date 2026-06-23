## ADDED Requirements

### Requirement: Get face by index

The system SHALL provide a Janet function `get-face` that extracts a face sub-shape from a parent shape by its 1-based stable index and returns it as an operable `rojcad/shape` abstract.

`(get-face shape index)` takes a ShapeData and an integer index. It SHALL return a ShapeData whose shape type is FACE. The returned ShapeData:
- SHALL NOT be registered in the viewer registry (invisible by default)
- SHALL be passable to `extrude`, `revolve`, `face-offset`, `face-fillet`, `face-chamfer`
- SHALL have a unique `shape_id` independent of the parent

#### Scenario: Extract a face from a box
- **WHEN** user calls `(get-face (box 10 20 30) 1)`
- **THEN** the system returns a ShapeData with shape type FACE

#### Scenario: Extrude an extracted face
- **WHEN** user calls `(extrude (get-face (box 10 20 30) 1) :h 5)`
- **THEN** the system returns a valid solid ShapeData

#### Scenario: Revolve an extracted face
- **WHEN** user calls `(revolve (get-face (cylinder :r 5 :h 10) 1) :ar 3.14 :dir [0 0 1])`
- **THEN** the system returns a valid solid ShapeData

#### Scenario: Error on out-of-range index
- **WHEN** user calls `(get-face (box 10) 999)`
- **THEN** the system signals an error

#### Scenario: Error on non-shape argument
- **WHEN** user calls `(get-face "not-a-shape" 1)`
- **THEN** the system signals an error

#### Scenario: Face index matches face-info index
- **WHEN** user calls `(-> (face-info shape) first :index (get-face shape))`
- **THEN** the system returns a valid ShapeData for the first face

### Requirement: Get edge by index

The system SHALL provide a Janet function `get-edge` that extracts an edge sub-shape from a parent shape by its 1-based stable index and returns it as an operable `rojcad/shape` abstract.

`(get-edge shape index)` takes a ShapeData and an integer index. It SHALL return a ShapeData whose shape type is EDGE. The returned ShapeData SHALL NOT be viewer-registered (invisible by default).

#### Scenario: Extract an edge from a box
- **WHEN** user calls `(get-edge (box 10 20 30) 1)`
- **THEN** the system returns a ShapeData with shape type EDGE

#### Scenario: Edge index matches edge-info index
- **WHEN** user calls `(-> (edge-info shape) first :index (get-edge shape))`
- **THEN** the system returns a valid ShapeData for the first edge

#### Scenario: Error on out-of-range index
- **WHEN** user calls `(get-edge (box 10) 999)`
- **THEN** the system signals an error

### Requirement: Convenience faces/edges functions

The system SHALL provide convenience Janet functions `faces` and `edges` that return an array of all face/edge ShapeData abstracts for a shape, combining query and extraction in one step.

`(faces shape)` SHALL be equivalent to mapping `get-face` over each face-info entry.
`(edges shape)` SHALL be equivalent to mapping `get-edge` over each edge-info entry.

#### Scenario: faces returns all face ShapeData
- **WHEN** user calls `(length (faces (box 10 20 30)))`
- **THEN** the system returns `6`

#### Scenario: edges returns all edge ShapeData
- **WHEN** user calls `(length (edges (box 10 20 30)))`
- **THEN** the system returns `12`

#### Scenario: faces are operable
- **WHEN** user calls `(extrude (first (faces (box 10 20 30))) :h 5)`
- **THEN** the system returns a valid solid ShapeData
