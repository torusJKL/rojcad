## ADDED Requirements

### Requirement: Solid fillet

The system SHALL provide a `fillet` function that rounds edges of a 3D shape by a given radius.

`(fillet shape :r radius)` operates on all edges.
`(fillet shape :r radius :e [idx1 idx2 ...])` operates on selected edges only.

The function SHALL accept a ShapeData with shape type SOLID or SHELL.
The function SHALL return a new ShapeData with the filleted shape.
The function SHALL error if radius <= 0.

#### Scenario: fillet all edges of a box
- **WHEN** user calls `(fillet (box 10 10 10) :r 2)`
- **THEN** the system returns a ShapeData with all 12 edges rounded

#### Scenario: fillet selected edges
- **WHEN** user calls `(fillet (box 10 10 10) :r 2 :e [0 1 2])`
- **THEN** the system returns a ShapeData with edges 0, 1, 2 rounded and other edges unchanged

#### Scenario: fillet errors on negative radius
- **WHEN** user calls `(fillet (box 10) :r -1)`
- **THEN** the system signals an error

#### Scenario: fillet errors on zero radius
- **WHEN** user calls `(fillet (box 10) :r 0)`
- **THEN** the system signals an error

#### Scenario: fillet errors on degenerate result
- **WHEN** user calls `(fillet (box 10 10 2) :r 5)` (radius larger than feasible)
- **THEN** the system signals an error "fillet: operation produced no geometry"

### Requirement: Solid chamfer

The system SHALL provide a `chamfer` function that bevels edges of a 3D shape by a given distance.

`(chamfer shape :d distance)` operates on all edges.
`(chamfer shape :d distance :e [idx1 idx2 ...])` operates on selected edges only.

The function SHALL accept a ShapeData with shape type SOLID or SHELL.
The function SHALL return a new ShapeData with the chamfered shape.
The function SHALL error if distance <= 0.

#### Scenario: chamfer all edges of a box
- **WHEN** user calls `(chamfer (box 10 10 10) :d 1)`
- **THEN** the system returns a ShapeData with all 12 edges beveled

#### Scenario: chamfer selected edges
- **WHEN** user calls `(chamfer (box 10 10 10) :d 1 :e [0 1 2])`
- **THEN** the system returns a ShapeData with edges 0, 1, 2 beveled

#### Scenario: chamfer errors on negative distance
- **WHEN** user calls `(chamfer (box 10) :d -1)`
- **THEN** the system signals an error

#### Scenario: chamfer errors on degenerate result
- **WHEN** user calls `(chamfer (box 10 10 3) :d 2)` (distance too large for edge length)
- **THEN** the system signals an error "chamfer: operation produced no geometry"

### Requirement: Support :eager and :hide keywords

`fillet` and `chamfer` SHALL support the standard `:eager` and `:hide` keywords consistent with other CAD operations.

#### Scenario: fillet with eager tessellation
- **WHEN** user calls `(fillet (box 10) :r 2 :eager)`
- **THEN** the result shape is tessellated immediately

#### Scenario: chamfer with hidden result
- **WHEN** user calls `(chamfer (box 10) :d 1 :hide)`
- **THEN** the result shape is not visible
