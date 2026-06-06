## ADDED Requirements

### Requirement: Query shape type

The system SHALL provide a function `shape-type` that returns the topological type of a shape as a Janet keyword.

The function SHALL accept one required positional argument that MUST be a `rojcad/shape` abstract value.

The function SHALL return a keyword corresponding to the shape's OCCT TopAbs_ShapeEnum value: `:solid`, `:face`, `:edge`, `:wire`, `:shell`, `:vertex`, `:compound`, `:compound-solid`, or `:shape`.

#### Scenario: Box is a solid
- **WHEN** user calls `(shape-type (make-box 10 10 10))`
- **THEN** the system returns `:solid`

#### Scenario: Sphere is a solid
- **WHEN** user calls `(shape-type (make-sphere 10))`
- **THEN** the system returns `:solid`

#### Scenario: Type error for non-shape
- **WHEN** user calls `(shape-type 42)`
- **THEN** the system SHALL signal a Janet type error

### Requirement: Check visibility

The system SHALL provide a function `visible?` that returns whether a shape's `visible` flag is set to true.

The function SHALL accept one required positional argument that MUST be a `rojcad/shape` abstract value.

The function SHALL return `true` if the shape's metadata flag `visible` is true, or `false` otherwise.

Newly created shapes SHALL have `visible = true` by default.

#### Scenario: New shape is visible
- **WHEN** user calls `(visible? (make-box 10 10 10))`
- **THEN** the system returns `true`

#### Scenario: Hidden shape is not visible
- **WHEN** user calls `(def s (make-box 10 10 10)) (hide s) (visible? s)`
- **THEN** the system returns `false`

### Requirement: Hide a shape

The system SHALL provide a function `hide` that sets a shape's `visible` flag to false by mutating the shape's metadata in place.

The function SHALL accept one required positional argument that MUST be a `rojcad/shape` abstract value.

The function SHALL return nil.

#### Scenario: Hide a shape
- **WHEN** user calls `(def s (make-box 10 10 10)) (hide s)`
- **THEN** the shape `s` has `visible = false`

### Requirement: Show a shape

The system SHALL provide a function `show` that sets a shape's `visible` flag to true by mutating the shape's metadata in place.

The function SHALL accept one required positional argument that MUST be a `rojcad/shape` abstract value.

The function SHALL return nil.

#### Scenario: Show a hidden shape
- **WHEN** user calls `(def s (make-box 10 10 10)) (hide s) (show s)`
- **THEN** the shape `s` has `visible = true`
