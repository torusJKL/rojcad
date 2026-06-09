## ADDED Requirements

### Requirement: All moved functions preserve behavior

Each function moved from C to Janet SHALL produce identical results before and after migration.

#### Scenario: Edge toggle returns boolean
- **WHEN** `(edge-toggle-inactive)` is called
- **THEN** it returns true or false indicating the new edge visibility state

#### Scenario: Projection toggle returns boolean
- **WHEN** `(projection-toggle)` is called
- **THEN** it returns true if now in perspective mode, false if orthographic

#### Scenario: Get/set functions work correctly
- **WHEN** `(stats-overlay true)` is called
- **THEN** the stats overlay becomes visible
- **WHEN** `(stats-overlay)` is called with no args
- **THEN** it returns true (the current state)

#### Scenario: Window size query returns tuple
- **WHEN** `(window-size?)` is called
- **THEN** it returns a tuple `[width height]` of positive integers

### Requirement: Visibility and query functions preserve behavior

Each function moved from C to Janet SHALL produce identical results before and after migration.

#### Scenario: Show/hide work correctly
- **WHEN** `(show shape)` is called
- **THEN** the shape becomes visible and nil is returned
- **WHEN** `(hide shape)` is called
- **THEN** the shape becomes hidden and nil is returned

#### Scenario: Variadic show/hide/purge work
- **WHEN** `(show shape-a shape-b)` is called
- **THEN** both shapes become visible

#### Scenario: Type queries return correct types
- **WHEN** `(shape-type box-shape)` is called
- **THEN** it returns `:solid`
- **WHEN** `(wire? wire-shape)` is called
- **THEN** it returns true

#### Scenario: Variadic queries return array
- **WHEN** `(visible? shape-a shape-b)` is called
- **THEN** it returns an array of boolean values

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

### Requirement: I/O functions preserve behavior

#### Scenario: write-step succeeds
- **WHEN** `(write-step shape "/tmp/test.step")` is called
- **THEN** a STEP file is written and nil is returned
- **WHEN** the path is invalid
- **THEN** an error is signaled

#### Scenario: read-step loads a STEP file
- **WHEN** `(read-step "/tmp/test.step")` is called
- **THEN** it returns a rojcad/shape abstract value

### Requirement: Sketch and wire functions preserve behavior

#### Scenario: Sketch pipeline works
- **WHEN** `(-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))` is called
- **THEN** it returns a Face shape

#### Scenario: Wire operations work
- **WHEN** `(wire-fillet wire :r 2)` is called
- **THEN** it returns a new Wire with rounded vertices

### Requirement: Medium-complexity functions preserve behavior

#### Scenario: Primitives with keywords work
- **WHEN** `(sphere 10 :c [1 2 3] :a 90)` is called
- **THEN** it returns a hemisphere shape centered at [1,2,3]

#### Scenario: Booleans chain correctly
- **WHEN** `(cut box-a box-b box-c)` is called
- **THEN** box-c is subtracted from (box-a minus box-b)

#### Scenario: Transforms preserve original
- **WHEN** `(translate shape 5 0 0)` is called
- **THEN** it returns a new shape at the translated position, the original is unchanged

#### Scenario: Text creation works
- **WHEN** `(text "Hello" "font.ttf" 10 :depth 5)` is called
- **THEN** it returns an extruded 3D text shape

### Requirement: Complex functions preserve behavior across all construction modes

#### Scenario: Box all modes
- **WHEN** `(box 10)` is called
- **THEN** it returns a 10x10x10 cube
- **WHEN** `(box 10 20 30)` is called
- **THEN** it returns a 10x20x30 box
- **WHEN** `(box :pl [0 0 0] :ph [10 20 30])` is called
- **THEN** it returns a box from opposite corners
- **WHEN** `(box 10 :c [5 5 5])` is called
- **THEN** it returns a centered cube

#### Scenario: Cylinder all modes
- **WHEN** `(cylinder 5 10)` is called
- **THEN** it returns a cylinder r=5 h=10 along Z
- **WHEN** `(cylinder :fp [0 0 0] :tp [0 0 10] :r 5)` is called
- **THEN** it returns a cylinder between two points

#### Scenario: Torus all modes
- **WHEN** `(torus 20 10)` is called
- **THEN** it returns a full torus
- **WHEN** `(torus 20 10 :a 180)` is called
- **THEN** it returns a half torus
