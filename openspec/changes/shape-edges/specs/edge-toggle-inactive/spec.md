## ADDED Requirements

### Requirement: Toggle inactive edge visibility from REPL

The system SHALL provide a Janet function `edge-toggle-inactive` that toggles the visibility of edges on non-selected shapes. The system SHALL also provide `edge-inactive-show?` to query the current state.

#### Scenario: Toggle hides inactive edges
- **WHEN** the user calls `(edge-toggle-inactive)` while inactive edges are visible
- **THEN** the call returns `false` and all edges of non-selected shapes disappear from the viewer

#### Scenario: Toggle shows inactive edges
- **WHEN** the user calls `(edge-toggle-inactive)` while inactive edges are hidden
- **THEN** the call returns `true` and edges of non-selected shapes reappear

#### Scenario: Query returns current state
- **WHEN** the user calls `(edge-inactive-show?)` after hiding inactive edges
- **THEN** the call returns `false`

#### Scenario: Toggle does not affect active edges
- **WHEN** the user calls `(edge-toggle-inactive)` to hide inactive edges
- **THEN** edges of the selected shape remain visible (if `edge-active-show?` is true)

#### Scenario: Toggle persists until changed
- **WHEN** the user hides inactive edges, then creates a new shape
- **THEN** the new shape's edges are also hidden until the toggle is flipped back
