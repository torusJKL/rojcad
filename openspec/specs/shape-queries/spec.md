## ADDED Requirements

### Requirement: Query selected shapes

The system SHALL provide a Janet function `selected-shapes` that returns a tuple of `ShapeData` abstract values currently selected in the 3D viewer. An empty selection SHALL return an empty tuple `()`.

#### Scenario: Returns selected shapes as ShapeData objects
- **WHEN** the user selects a shape in the 3D viewer
- **THEN** `(selected-shapes)` returns a tuple containing that shape's `ShapeData` abstract value
- **AND** the returned value is the actual Janet GC-allocated object, passable to `hide`, `show`, `visible?`, and `shape-type`

#### Scenario: Multiple shapes selected
- **WHEN** the user selects multiple shapes via Shift+click or Ctrl+click
- **THEN** `(selected-shapes)` returns a tuple containing all selected `ShapeData` objects

#### Scenario: Empty selection
- **WHEN** no shapes are selected in the viewer
- **THEN** `(selected-shapes)` returns an empty tuple `()`

#### Scenario: Iteration over selected shapes
- **WHEN** `(selected-shapes)` returns shapes
- **THEN** Janet `each`, `map`, and `filter` work on the returned tuple
- **AND** operations like `(each s (selected-shapes) (hide s))` succeed

### Requirement: List all registered shapes

The system SHALL provide a Janet function `list-shapes` that returns a tuple of all `ShapeData` abstract values currently registered in the viewer's `ShapeRegistry`. Registration happens when a shape is created with `show` or the `display` helper. Shapes that were created and never shown are not registered and SHALL NOT appear in the result.

#### Scenario: Returns all registered shapes
- **WHEN** the user calls `(list-shapes)` with no arguments
- **THEN** the system returns a tuple of all `ShapeData` objects in the `ShapeRegistry`

#### Scenario: Empty registry
- **WHEN** no shapes have been registered in the viewer
- **THEN** `(list-shapes)` returns an empty tuple `()`

#### Scenario: Unshown shapes excluded
- **WHEN** a shape was created with `:hide` and never shown
- **THEN** it SHALL NOT appear in the result of `(list-shapes)`

#### Scenario: Purged shapes excluded
- **WHEN** a shape was removed from the registry via `purge`
- **THEN** it SHALL NOT appear in the result of `(list-shapes)`

### Requirement: Filter by visibility

The `list-shapes` function SHALL accept optional keyword arguments `:visible` and `:hidden` to filter the returned shapes by their current visibility state. If neither keyword is provided, all registered shapes are returned. If both are provided, `:hidden` takes precedence.

#### Scenario: Filter visible shapes
- **WHEN** the user calls `(list-shapes :visible)`
- **THEN** only shapes with `visible = true` in the registry are returned

#### Scenario: Filter hidden shapes
- **WHEN** the user calls `(list-shapes :hidden)`
- **THEN** only shapes with `visible = false` (hidden) in the registry are returned

#### Scenario: Both keywords provided
- **WHEN** the user calls `(list-shapes :visible :hidden)`
- **THEN** the system treats it as `:hidden` (precedence rule) and returns only hidden shapes

#### Scenario: Iteration over filtered list
- **WHEN** `(list-shapes :visible)` returns visible shapes
- **THEN** `(each s (list-shapes :visible) (hide s))` hides all visible shapes
