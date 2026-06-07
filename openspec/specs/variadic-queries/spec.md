## ADDED Requirements

### Requirement: Query functions accept multiple shapes and return tuple
`shape-type`, `visible?`, `wire?`, `face?`, and `solid?` SHALL accept one or more shape arguments. They SHALL return a tuple of results, one per shape, in the order the shapes were provided. For a single shape argument, they SHALL return a single-element tuple.

#### Scenario: shape-type with multiple shapes
- **WHEN** `(shape-type box sphere)` is called with a box and a sphere
- **THEN** the result is `@[:solid :solid]`

#### Scenario: visible? with multiple shapes
- **WHEN** `(visible? a b)` is called where `a` is visible and `b` is hidden
- **THEN** the result is `@[true false]`

#### Scenario: wire? with multiple shapes
- **WHEN** `(wire? wire-a face-b)` is called with a wire and a face
- **THEN** the result is `@[true false]`

#### Scenario: face? with multiple shapes
- **WHEN** `(face? face-a wire-b)` is called with a face and a wire
- **THEN** the result is `@[true false]`

#### Scenario: solid? with multiple shapes
- **WHEN** `(solid? solid-a face-b)` is called with a solid and a face
- **THEN** the result is `@[true false]`

#### Scenario: Single shape returns single-element tuple
- **WHEN** `(shape-type (box 10))` is called with one shape
- **THEN** the result is `@[:solid]` (a tuple, not a bare keyword)

#### Scenario: Zero shapes returns empty tuple
- **WHEN** `(shape-type)` is called with no arguments
- **THEN** the result is `@[]`
