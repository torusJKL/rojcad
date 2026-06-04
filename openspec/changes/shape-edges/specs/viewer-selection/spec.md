## MODIFIED Requirements

### Requirement: Visual highlighting

**Change**: The selected shape's edges now render in light blue instead of "brighter/thicker".

The selected shape SHALL be visually distinguished from unselected shapes.

#### Scenario: Selected shape tinted
- **WHEN** a shape is selected
- **THEN** its mesh surface is tinted blue and its edges become light blue

#### Scenario: Highlight removed on deselection
- **WHEN** selection is cleared
- **THEN** the previously selected shape returns to its normal appearance (grey mesh, light grey edges)

#### Scenario: Active edge toggle respected
- **WHEN** the user has hidden active edges via `(edge-toggle-active)` and selects a shape
- **THEN** the shape's mesh still tints blue but its edges remain hidden until the toggle is flipped back
