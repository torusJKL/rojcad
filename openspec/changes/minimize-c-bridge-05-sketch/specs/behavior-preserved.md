## ADDED Requirements

### Requirement: Sketch and wire functions preserve behavior

#### Scenario: Sketch pipeline works
- **WHEN** `(-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))` is called
- **THEN** it returns a Face shape

#### Scenario: Wire operations work
- **WHEN** `(wire-fillet wire :r 2)` is called
- **THEN** it returns a new Wire with rounded vertices
