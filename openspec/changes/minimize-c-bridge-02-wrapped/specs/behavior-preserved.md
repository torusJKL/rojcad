## ADDED Requirements

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
