## ADDED Requirements

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
