## ADDED Requirements

### Requirement: Click selection

The viewer SHALL select a shape when the user clicks on it. Selection uses CPU ray-triangle intersection against all visible shapes.

#### Scenario: Click on shape selects it
- **WHEN** the user left-clicks on a visible shape
- **THEN** that shape becomes the selected shape (previous selection is cleared)

#### Scenario: Click on empty space deselects
- **WHEN** the user left-clicks on empty space (no shape intersection)
- **THEN** any current selection is cleared

#### Scenario: Nearest shape is selected
- **WHEN** the user clicks on overlapping shapes
- **THEN** the shape with the closest intersection point along the ray is selected

#### Scenario: Occluded shape not selectable
- **WHEN** a shape is fully behind another shape
- **THEN** clicking the visible area selects the front shape

### Requirement: Visual highlighting

The selected shape SHALL be visually distinguished from unselected shapes.

#### Scenario: Selected shape tinted
- **WHEN** a shape is selected
- **THEN** its mesh surface is tinted blue and its edges become brighter/thicker

#### Scenario: Highlight removed on deselection
- **WHEN** selection is cleared
- **THEN** the previously selected shape returns to its normal appearance

### Requirement: Selection feedback to REPL

The viewer SHALL send selection events back to the Janet REPL via a channel.

#### Scenario: Selection triggers callback
- **WHEN** a shape is selected in the viewer
- **THEN** the REPL receives a notification with the selected shape's ID
