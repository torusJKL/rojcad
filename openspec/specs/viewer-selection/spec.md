## Requirements

### Requirement: View supports multi-shape selection

The 3D viewer SHALL support selecting multiple shapes simultaneously using keyboard modifiers. Selection state SHALL be maintained as a set of shape IDs, rendered with highlight and active edges for all selected shapes.

#### Scenario: Plain click selects a single shape
- **WHEN** user clicks on a shape without holding Ctrl or Shift
- **THEN** the clicked shape becomes the only selected shape
- **AND** all previously selected shapes are deselected

#### Scenario: Shift+click adds to selection
- **WHEN** user holds Shift and clicks on a shape that is not currently selected
- **THEN** the clicked shape is added to the selection set
- **AND** all previously selected shapes remain selected

#### Scenario: Shift+click on already-selected shape is a no-op
- **WHEN** user holds Shift and clicks on a shape that is already selected
- **THEN** the selection set is unchanged

#### Scenario: Ctrl+click toggles selection
- **WHEN** user holds Ctrl and clicks on a shape that is not currently selected
- **THEN** the clicked shape is added to the selection set

#### Scenario: Ctrl+click deselects
- **WHEN** user holds Ctrl and clicks on a shape that is currently selected
- **THEN** the clicked shape is removed from the selection set

#### Scenario: Plain click on empty space clears selection
- **WHEN** user clicks on empty space without holding Ctrl or Shift
- **THEN** all shapes are deselected

#### Scenario: Ctrl+click on empty space is a no-op
- **WHEN** user holds Ctrl and clicks on empty space
- **THEN** the selection set is unchanged

#### Scenario: Shift+click on empty space is a no-op
- **WHEN** user holds Shift and clicks on empty space
- **THEN** the selection set is unchanged

#### Scenario: Nearest shape is selected
- **WHEN** user clicks on overlapping shapes
- **THEN** the shape with the closest intersection point along the ray is selected

#### Scenario: Occluded shape not selectable
- **WHEN** a shape is fully behind another shape
- **THEN** clicking the visible area selects the front shape

### Requirement: Visual highlighting

The viewer SHALL visually distinguish selected shapes from unselected shapes using distinct rendering pipelines.

#### Scenario: All selected shapes highlighted
- **WHEN** one or more shapes are selected
- **THEN** every selected shape renders with the blue highlight pipeline
- **AND** every selected shape renders with active (blue) and brighter/thicker edges

#### Scenario: Non-selected shape rendering
- **WHEN** a shape is not selected
- **THEN** its mesh surface renders with the default gray pipeline
- **AND** its edges render as inactive (gray)

#### Scenario: Highlight removed on deselection
- **WHEN** a shape is deselected or selection is cleared
- **THEN** the shape returns to its non-selected appearance

### Requirement: Selection fires on mouse release, not press

The 3D viewer SHALL trigger selection changes on mouse button release, not press. A drag threshold SHALL distinguish clicks from drags to prevent accidental selection during camera orbit.

#### Scenario: Click selects on release
- **WHEN** user presses and releases the left mouse button without significant movement
- **THEN** the selection change is applied on release

#### Scenario: Drag does not trigger selection
- **WHEN** user presses the left mouse button, moves the cursor more than 3px, then releases
- **THEN** no selection change occurs
- **AND** the camera orbits during the drag

### Requirement: Selection events are communicated to Janet

The viewer SHALL report selection changes to Janet via the existing `poll-selection` mechanism, enhanced to support toggle-off and multi-select events.

#### Scenario: Shape toggled on is reported
- **WHEN** a shape is added to the selection (via plain, Shift, or Ctrl click)
- **THEN** `poll-selection` returns the shape ID as a number

#### Scenario: Shape toggled off is reported
- **WHEN** a shape is removed from selection via Ctrl+click toggle-off
- **THEN** `poll-selection` returns `[:deselected <shape_id>]`

#### Scenario: Selection cleared is reported
- **WHEN** the entire selection is cleared (plain click on empty space)
- **THEN** `poll-selection` returns `:deselected`

#### Scenario: No event returns nil
- **WHEN** no selection change has occurred since the last poll
- **THEN** `poll-selection` returns `nil`

### Requirement: `on-select` callback receives individual events

The `on-select` callback registered via Janet SHALL be invoked for each selection event (toggle-on, toggle-off, cleared).

#### Scenario: Callback receives toggled-on event
- **WHEN** a shape is selected and an `on-select` callback is registered
- **THEN** the callback is invoked with the shape ID as a number

#### Scenario: Callback receives cleared event
- **WHEN** selection is cleared and an `on-select` callback is registered
- **THEN** the callback is invoked with `nil`
