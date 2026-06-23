## ADDED Requirements

### Requirement: Enriched edge metadata

The system SHALL extend the existing `edge-info` function to include additional fields in each returned struct: `:length`, `:radius`, `:axis`, `:center`.

`(edge-info shape)` returns an array of structs. Each struct SHALL contain:
- `:index` (integer, 1-based) — stable index for use with `get-edge`
- `:type` (string) — curve type: `"line"`, `"circle"`, `"ellipse"`, `"hyperbola"`, `"parabola"`, `"bezier-curve"`, `"bspline-curve"`, `"offset-curve"`, `"other-curve"`
- `:length` (number) — curve length
- `:start` (tuple [x y z]) — first point of the edge
- `:end` (tuple [x y z]) — last point of the edge
- `:radius` (number) — radius for circular/elliptical edges; `0.0` for lines and other types
- `:axis` (tuple [x y z]) — direction for lines, axis for circles/ellipses; `(0 0 0)` for other types
- `:center` (tuple [x y z]) — center point for circles/ellipses; midpoint for lines; `(0 0 0)` for other types

#### Scenario: Enriched edge info for a box
- **WHEN** user calls `(edge-info (box 10 20 30))`
- **THEN** the system returns an array of 12 structs
- **AND** each struct has `:type` equal to `"line"`
- **AND** each struct has `:length` greater than 0
- **AND** each struct has `:radius` equal to `0.0` (lines have no radius)

#### Scenario: Enriched edge info for a cylinder
- **WHEN** user calls `(edge-info (cylinder :r 5 :h 10))`
- **THEN** the system returns an array of structs
- **AND** exactly 2 structs have `:type` equal to `"circle"` with `:radius` equal to `5.0`
- **AND** the remaining structs have `:type` equal to `"line"`

#### Scenario: Circle edge has axis and center
- **WHEN** user calls `(edge-info (cylinder :r 5 :h 10))`
- **THEN** the circle edges have `:axis` equal to `(0 0 1)` or `(0 0 -1)`
- **AND** their `:center` has z-coordinate at the cylinder end (0 or 10)

#### Scenario: Line edge has axis
- **WHEN** user calls `(edge-info (box 10 20 30))`
- **THEN** line edges have a unit `:axis` vector pointing along the edge direction

#### Scenario: Backward compatibility
- **WHEN** existing code calls `(edge-info shape)` and accesses `:index`, `:type`, `:start`, `:end`
- **THEN** those fields behave identically to before
