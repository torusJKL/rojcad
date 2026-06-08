## ADDED Requirements

### Requirement: Side-effect functions accept multiple shapes
`hide`, `show`, `purge`, and `registry-remove` SHALL accept zero or more shape arguments. When called with multiple shapes, the effect SHALL be applied to each shape in sequence. When called with zero shapes, the function SHALL return nil with no side effects.

#### Scenario: Hide multiple shapes
- **WHEN** `(hide a b c)` is called with three registered shapes
- **THEN** all three shapes are no longer visible in the viewer

#### Scenario: Show multiple shapes
- **WHEN** `(show a b c)` is called with three hidden shapes
- **THEN** all three shapes become visible in the viewer

#### Scenario: Purge multiple shapes
- **WHEN** `(purge a b c)` is called with three registered shapes
- **THEN** all three shapes are removed from the viewer registry

#### Scenario: Zero shapes returns nil
- **WHEN** `(hide)` is called with no arguments
- **THEN** the result is nil and no error is signaled

#### Scenario: Single shape still works
- **WHEN** `(hide a)` is called with one shape
- **THEN** the shape is hidden, identical to the current single-arg behavior
