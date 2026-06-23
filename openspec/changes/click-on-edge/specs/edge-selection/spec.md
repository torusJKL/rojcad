## ADDED Requirements

### Requirement: Edge picked on viewer click

The system SHALL detect when a user clicks on a rendered edge in the 3D viewer and emit an edge selection event with the shape ID and edge index.

#### Scenario: Click on visible edge
- **WHEN** user clicks on a rendered edge of a shape in the viewer
- **THEN** the system emits an edge-selected event containing the shape ID and the topological edge index (0-based)

#### Scenario: Click on empty space near shape
- **WHEN** user clicks within `EDGE_THICKNESS / 2 + 2` screen pixels of a projected edge polyline
- **THEN** the system SHALL treat this as clicking the edge (edge wins over mesh surface)

#### Scenario: Edge overlap at corners
- **WHEN** user clicks near a vertex where multiple edges meet
- **THEN** the system SHALL select the edge whose projected segment is closest to the camera (minimum z-depth after perspective divide)

#### Scenario: Click on synthetic edge
- **WHEN** user clicks on a synthetic wireframe edge (equator/meridian of sphere, etc.)
- **THEN** the system SHALL NOT emit an edge selection event

### Requirement: Multi-edge selection with modifiers

The system SHALL support multi-edge selection using the same modifier conventions as shape selection.

#### Scenario: Plain click replaces edge selection
- **WHEN** user clicks on an edge without modifiers
- **THEN** the system clears the edge selection set and selects only the clicked edge

#### Scenario: Shift+click adds edge
- **WHEN** user Shift+clicks on an edge
- **THEN** the system adds the edge to the selection set (if not already present)

#### Scenario: Ctrl+click toggles edge
- **WHEN** user Ctrl+clicks on an edge
- **THEN** the system toggles the edge in/out of the selection set

#### Scenario: Click on empty space clears edges
- **WHEN** user clicks on empty space (no edge, no shape) without modifiers
- **THEN** the system clears the edge selection set and clears shape selection

#### Scenario: Edge deselected emits event
- **WHEN** a previously selected edge is deselected via Ctrl+click or clear
- **THEN** the system emits an edge-deselected event with the shape ID and edge index

### Requirement: Edge selection also selects parent shape

When an edge is selected, its parent shape SHALL also appear in the shape selection set.

#### Scenario: Single edge click selects shape
- **WHEN** user clicks on an edge of a shape that is not currently selected
- **THEN** the shape is added to the shape selection set (and visually highlighted)

#### Scenario: Second edge of same shape
- **WHEN** user Shift+clicks another edge of the same shape
- **THEN** the shape remains in the shape selection set (no duplicate entry)

#### Scenario: Edge deselected does not deselect shape
- **WHEN** user Ctrl+clicks an edge to deselect it, but the shape still has other selected edges
- **THEN** the shape remains in the shape selection set

#### Scenario: All edges deselected clears shape only on last
- **WHEN** the last edge of a shape is deselected
- **THEN** the shape is removed from the shape selection set (or kept if also explicitly selected — shape-only selection takes precedence)

### Requirement: Edge selection query API

The system SHALL provide a Janet function `(edge-selection)` that returns the current edge selection state.

#### Scenario: Query with active edge selection
- **WHEN** one or more edges are selected and user calls `(edge-selection)`
- **THEN** the return value is an array of structs, each with `:sid` (numeric shape id), `:shape`, `:name` (or nil), and `:edges` (set of edge indices), grouped by shape

#### Scenario: Query with no edges selected
- **WHEN** no edges are selected and user calls `(edge-selection)`
- **THEN** the return value is an empty array `@[]`

### Requirement: Named shapes in REPL output

The system SHALL display the `def`-bound variable name of a shape in selection event output, instead of the opaque abstract pointer.

#### Scenario: Named shape selection
- **WHEN** user clicks an edge of a shape that was defined with `(def b (box 10))`
- **THEN** REPL output SHALL display `edge 3 of b selected`

#### Scenario: Unnamed shape selection
- **WHEN** user clicks an edge of a shape that was NOT defined with `def` (e.g., result of a compound expression)
- **THEN** REPL output SHALL display `edge 3 of <shape {id}> selected`

#### Scenario: Shape selection (not edge)
- **WHEN** user clicks a shape surface (no edge hit)
- **THEN** REPL output SHALL display `b selected` (using the name if available)

### Requirement: Hidden edge selection respects back-edge toggle

The system SHALL only allow selecting hidden (back-facing/occluded) edges when `(edge-hidden true)` is active. When back edges are hidden, clicking on an occluded edge SHALL be treated as a miss.

#### Scenario: Click on visible edge with back edges disabled
- **WHEN** back edges are disabled (default) and user clicks on a front-facing edge
- **THEN** the edge is selected (normal behavior)

#### Scenario: Click on hidden edge with back edges disabled
- **WHEN** back edges are disabled and user clicks on an occluded edge (behind the mesh surface)
- **THEN** the click is treated as a miss — no edge selection event is emitted and shape picking fallback applies

#### Scenario: Click on hidden edge with back edges enabled
- **WHEN** back edges are enabled (`(edge-hidden true)` or **X** key) and user clicks on an occluded edge
- **THEN** the edge is selected (hidden edges are pickable)

#### Scenario: Occlusion detection by ray-mesh intersection
- **WHEN** user clicks near an edge polyline
- **THEN** the system casts a ray from the camera through the edge midpoint and checks if the same shape's mesh surface is closer than the edge midpoint
- **AND** if a mesh triangle is closer (by > 0.1 world units), the edge is considered occluded/hidden

### Requirement: Convenience fillet/chamfer helpers

The system SHALL provide convenience functions `(fillet-selected ...)` and `(chamfer-selected ...)` that operate on the current edge selection.

#### Scenario: Fillet selected edges
- **WHEN** user calls `(fillet-selected :r 2)` with one or more edges selected
- **THEN** the system applies fillet with radius 2 to all selected edges, grouped by their parent shape

#### Scenario: No edges selected
- **WHEN** user calls `(fillet-selected :r 2)` with no edges selected
- **THEN** the system signals an error "no edge selected"

#### Scenario: Chamfer selected edges
- **WHEN** user calls `(chamfer-selected :d 1)` with one or more edges selected
- **THEN** the system applies chamfer with distance 1 to all selected edges, grouped by their parent shape
