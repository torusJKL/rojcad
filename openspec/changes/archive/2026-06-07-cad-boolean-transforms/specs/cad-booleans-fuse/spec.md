## ADDED Requirements

### Requirement: Fuse (union) two shapes

The system SHALL provide a function `fuse` that performs a boolean union operation using the OCCT BRepAlgoAPI_Fuse kernel.

The function SHALL accept two required positional arguments, both of which MUST be `rojcad/shape` abstract values. The two operand shapes are combined into a single solid.

The function SHALL return a new `rojcad/shape` abstract value representing the union of both input shapes. The original operand shapes SHALL remain unchanged.

If the boolean operation fails (e.g., produces an invalid result), the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Fuse two overlapping boxes
- **WHEN** user calls `(def a (box 10 10 10)) (def b (box 10 10 10 :c [5 5 5])) (fuse a b)`
- **THEN** the system returns a shape representing the union of both boxes

#### Scenario: Fuse non-overlapping shapes
- **WHEN** user calls `(fuse (box 10 10 10) (box 10 10 10 :c [100 0 0]))`
- **THEN** the system returns a COMPOUND shape containing both boxes

#### Scenario: Fuse is commutative
- **WHEN** user calls `(def a (sphere 10)) (def b (box 10 10 10))`
- **THEN** `(fuse a b)` SHALL produce a result equivalent to `(fuse b a)`

#### Scenario: Original shapes preserved
- **WHEN** user calls `(def a (box 10 10 10)) (def b (sphere 10)) (def result (fuse a b))`
- **THEN** `a` and `b` SHALL remain valid shapes and `(visible? a)` SHALL return true

#### Scenario: Type checking
- **WHEN** user calls `(fuse 42 (sphere 10))`
- **THEN** the system SHALL signal a Janet type error
