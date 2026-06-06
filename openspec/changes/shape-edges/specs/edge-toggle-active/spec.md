## ADDED Requirements

### Requirement: Toggle active edge visibility from REPL

The system SHALL provide a Janet function `edge-toggle-active` that toggles the visibility of edges on the selected shape. The system SHALL also provide `edge-active-show?` to query the current state.

#### Scenario: Toggle hides active edges
- **WHEN** the user calls `(edge-toggle-active)` while active edges are visible
- **THEN** the call returns `false` and edges of the selected shape disappear while other edges remain

#### Scenario: Toggle shows active edges
- **WHEN** the user calls `(edge-toggle-active)` while active edges are hidden
- **THEN** the call returns `true` and edges of the selected shape reappear

#### Scenario: Query returns current state
- **WHEN** the user calls `(edge-active-show?)` after hiding active edges
- **THEN** the call returns `false`

#### Scenario: No active edges when nothing selected
- **WHEN** no shape is selected in the viewer and active edges are set to show
- **THEN** no active edges are rendered (there is no selected shape to draw edges for)
