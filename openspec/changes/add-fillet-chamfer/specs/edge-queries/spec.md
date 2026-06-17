## ADDED Requirements

### Requirement: Edge info

The system SHALL provide an `edge-info` function that returns metadata about all edges of a shape.

`(edge-info shape)` returns an array of tables, one per edge, each containing:
- `:index` (number) — 0-based edge index
- `:type` (string) — curve type: "line", "circle", "ellipse", "hyperbola", "parabola", "bezier-curve", "bspline-curve", "offset-curve", "other-curve"
- `:start` (tuple [x y z]) — start point of the edge in 3D space
- `:end` (tuple [x y z]) — end point of the edge in 3D space

#### Scenario: edge-info returns metadata for a box
- **WHEN** user calls `(edge-info (box 10 10 10))`
- **THEN** the system returns an array of 12 tables
- **AND** each table has keys :index, :type, :start, :end
- **AND** all :type values are "line"

#### Scenario: edge-info for a cylinder
- **WHEN** user calls `(edge-info (cylinder 5 10))`
- **THEN** the system returns an array of edge info tables
- **AND** some :type values are "circle" (top/bottom) and some are "line" (vertical)

#### Scenario: edge-info errors on non-shape input
- **WHEN** user calls `(edge-info nil)`
- **THEN** the system signals an error
