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
