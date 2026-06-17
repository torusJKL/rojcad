## ADDED Requirements

### Requirement: Highlight edges in viewer

The system SHALL provide a `highlight-edge` function that visually highlights specific edges of a shape in the viewer.

`(highlight-edge shape & indices)` highlights the specified edge indices of the shape.
Highlighted edges SHALL be rendered in a distinct color (orange) to distinguish from shape-level selection (blue).

#### Scenario: highlight edges 0 and 2 of a box
- **WHEN** user calls `(highlight-edge (box 10) 0 2)`
- **THEN** edges 0 and 2 of the box are rendered in orange

### Requirement: Clear edge highlights

The system SHALL provide a `highlight-edge-clear` function that removes all edge highlights from the viewer.

`(highlight-edge-clear)` removes all per-edge highlighting.

#### Scenario: clear all edge highlights
- **WHEN** user calls `(highlight-edge-clear)`
- **THEN** all edge highlights are removed from the viewer
