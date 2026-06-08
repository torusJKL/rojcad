## ADDED Requirements

### Requirement: Top-level `def` auto-shows shapes
When a shape is the result of a top-level `(def name expr)` form in the REPL, the system SHALL automatically register it in the viewer and make it visible, provided the shape's visible flag is `true` after construction.

#### Scenario: def-bound box appears in viewer
- **WHEN** the user evaluates `(def b (box 10))`
- **THEN** the shape `b` is registered in the viewer and visible

#### Scenario: Non-def shape expression is not shown
- **WHEN** the user evaluates `(box 10)` without a `def`
- **THEN** the shape is returned but NOT registered in the viewer

#### Scenario: Intermediate shape in compound expression is not shown
- **WHEN** the user evaluates `(translate (box 10) 5 0 0)`
- **THEN** only the outer result is visible; the inner `(box 10)` intermediate shape is NOT registered in the viewer

#### Scenario: def-bound shape with :hide is not shown
- **WHEN** the user evaluates `(def b (box 10 :hide))`
- **THEN** the shape `b` is bound but NOT registered in the viewer

#### Scenario: Hidden shape can be shown later
- **WHEN** the user evaluates `(def b (box 10 :hide))` followed by `(show b)`
- **THEN** the shape `b` becomes visible in the viewer

#### Scenario: def of non-shape value has no side effect
- **WHEN** the user evaluates `(def x 42)`
- **THEN** the value `42` is bound to `x` and no viewer operation occurs

### Requirement: `:hide` keyword on all shape constructors
Every shape-constructing JANET_FN (box, sphere, cylinder, cone, torus, cut, common, fuse, translate, rotate, scale, mirror) SHALL accept an optional `:hide` keyword. When present, the constructed shape SHALL have `visible = false`.

#### Scenario: box with :hide
- **WHEN** the user evaluates `(box 10 :hide)`
- **THEN** the returned shape has `visible?` returning `false`

#### Scenario: sphere with :hide
- **WHEN** the user evaluates `(sphere 10 :hide)`
- **THEN** the returned shape has `visible?` returning `false`

#### Scenario: boolean op with :hide
- **WHEN** the user evaluates `(cut a b :hide)`
- **THEN** the returned cut result has `visible?` returning `false`

#### Scenario: transform with :hide
- **WHEN** the user evaluates `(translate a 5 0 0 :hide)`
- **THEN** the returned translated shape has `visible?` returning `false`

#### Scenario: :hide with :eager works correctly
- **WHEN** the user evaluates `(box 10 :eager :hide)`
- **THEN** the shape is tessellated eagerly but has `visible?` returning `false`

### Requirement: `display` C function removed
The `display` JANET_FN SHALL be removed from the C bridge. Any call to `(display shape)` SHALL produce an error.

#### Scenario: display produces error
- **WHEN** the user evaluates `(display (box 10))`
- **THEN** the system signals an error: function not found or similar

#### Scenario: def-no-display is correct migration
- **WHEN** the user evaluates `(def b (box 10))` instead of `(def b (display (box 10)))`
- **THEN** the behavior is identical — shape is created, bound, and visible
