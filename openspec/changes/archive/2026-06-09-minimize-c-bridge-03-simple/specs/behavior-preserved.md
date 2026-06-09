## ADDED Requirements

### Requirement: Selection and view functions preserve behavior

#### Scenario: quit-requested returns boolean
- **WHEN** `(quit-requested)` is called
- **THEN** it returns true or false

#### Scenario: poll-selection returns correct types
- **WHEN** no selection event
- **THEN** `(poll-selection)` returns nil
- **WHEN** a shape is selected
- **THEN** `(poll-selection)` returns the shape ID
- **WHEN** selection is cleared
- **THEN** `(poll-selection)` returns `:deselected`

#### Scenario: selected-shapes returns tuple
- **WHEN** `(selected-shapes)` is called with no selection
- **THEN** it returns `()`
- **WHEN** shapes are selected
- **THEN** it returns a tuple of shape abstract values

#### Scenario: view-fit accepts shapes
- **WHEN** `(view-fit shape-a shape-b :reset)` is called
- **THEN** the camera animates to frame both shapes
