## ADDED Requirements

### Requirement: User can set a shape's color
The system SHALL provide a `color` function that sets a shape's render color. `(color shape [r g b])` SHALL mutate the shape's color metadata in place and return the same shape. Values SHALL be clamped to [0, 1].

#### Scenario: Set shape color
- **WHEN** user calls `(def s (sphere 5))` then `(color s [0.8 0.2 0.2])`
- **THEN** shape `s` SHALL have its color set to coral (r=0.8, g=0.2, b=0.2)

#### Scenario: Color returns the same shape
- **WHEN** user calls `(def s (sphere 5))` then `(def r (color s [1 0 0]))`
- **THEN** `r` SHALL be the same value as `s` (identical shape identity)

#### Scenario: Values clamped to [0, 1]
- **WHEN** user calls `(color s [2 -0.5 0])`
- **THEN** the stored color SHALL be `[1 0 0]` (clamped)

### Requirement: User can read a shape's color
The system SHALL provide a `get-color` function that returns the shape's current color as a 3-element tuple, or `nil` if no color has been set.

#### Scenario: Get color after setting
- **WHEN** user calls `(color (sphere 5) [0 1 0])` then `(get-color (sphere 5))`
- **THEN** the result SHALL be `@[0 1 0]`

#### Scenario: Get color returns nil when unset
- **WHEN** user calls `(get-color (sphere 5))` on a shape that has not had its color set
- **THEN** the result SHALL be `nil`

### Requirement: Viewer renders shapes with their assigned color
The viewer SHALL render each shape using its assigned color instead of hardcoded grey. When a shape has no color set, it SHALL render in the default grey (0.75, 0.75, 0.75).

#### Scenario: Shape renders in assigned color
- **WHEN** a shape has `color = [1 0 0]`
- **THEN** the viewer SHALL render its surface in red

#### Scenario: Default grey for uncolored shapes
- **WHEN** a shape has `color = None`
- **THEN** the viewer SHALL render its surface in grey (0.75, 0.75, 0.75)

### Requirement: Selection highlight overrides shape color
When a shape is selected, it SHALL render in the selection highlight blue regardless of its assigned color.

#### Scenario: Selected shape shows blue
- **WHEN** a shape with `color = [1 0 0]` is selected
- **THEN** the viewer SHALL render it in blue (0.3, 0.5, 1.0), not red
