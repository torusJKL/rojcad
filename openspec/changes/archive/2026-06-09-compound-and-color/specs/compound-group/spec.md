## ADDED Requirements

### Requirement: User can group shapes into an OCCT Compound
The system SHALL provide a `compound` function that accepts 1 or more shapes and returns their OCCT Compound wrapper. `(compound a)` SHALL return shape `a` unchanged. `(compound)` with 0 arguments SHALL signal an error.

#### Scenario: Group two shapes into a compound
- **WHEN** user calls `(compound (sphere 5) (box 10))`
- **THEN** the result SHALL be a single shape of type `COMPOUND` containing both the sphere and the box

#### Scenario: Single shape pass-through
- **WHEN** user calls `(compound (sphere 5))`
- **THEN** the result SHALL be the same sphere shape (not wrapped in a compound)

#### Scenario: Zero shapes errors
- **WHEN** user calls `(compound)`
- **THEN** an error SHALL be signaled

### Requirement: compound accepts :color keyword
The `compound` function SHALL accept an optional `:color` keyword followed by a 3-element tuple `[r g b]` with values in [0, 1]. When provided, the compound SHALL be created with that color set.

#### Scenario: Group with color
- **WHEN** user calls `(compound (sphere 5) (box 10) :color [1 0 0])`
- **THEN** the result SHALL be a red compound shape containing both primitives

#### Scenario: get-color returns the compound's color
- **WHEN** user calls `(get-color (compound (sphere 5) :color [0 1 0]))`
- **THEN** the result SHALL be `@[0 1 0]`

### Requirement: compound supports standard keywords
The `compound` function SHALL accept `:eager` (tessellate immediately) and `:hide` (suppress automatic show) keywords, following the same convention as other shape constructors.

#### Scenario: Eager tessellation
- **WHEN** user calls `(compound (sphere 5) (box 10) :eager)`
- **THEN** the compound SHALL be tessellated immediately (visible in viewer without further operation)

#### Scenario: Hidden creation
- **WHEN** user calls `(compound (sphere 5) (box 10) :hide)`
- **THEN** the compound SHALL NOT be shown in the viewer until `show` is called
