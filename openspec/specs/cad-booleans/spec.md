## ADDED Requirements

### Requirement: Cut (subtract) one shape from another

The system SHALL provide a function `cut` that performs a boolean subtraction operation using the OCCT BRepAlgoAPI_Cut kernel.

The function SHALL accept two required positional arguments, both of which MUST be `rojcad/shape` abstract values. The first argument is the target shape; the second is the tool shape being subtracted.

The function SHALL return a new `rojcad/shape` abstract value representing the resulting solid. The original operand shapes SHALL remain unchanged.

If the boolean operation fails (e.g., shapes do not intersect or produce an invalid result), the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Cut a sphere from a box
- **WHEN** user calls `(def box (make-box 20 20 20)) (def sphere (make-sphere 10)) (cut box sphere)`
- **THEN** the system returns a shape representing the box with the spherical volume removed

#### Scenario: Non-overlapping shapes
- **WHEN** user calls `(cut (make-box 10 10 10) (make-box 10 10 10 :center '(100 0 0)))`
- **THEN** the system SHALL signal a Janet error indicating the cut produced an empty result

#### Scenario: Cut is non-commutative
- **WHEN** user calls `(def a (make-box 10 10 10)) (def b (make-box 5 5 5 :center '(2 2 2)))`
- **THEN** `(cut a b)` SHALL produce a different result than `(cut b a)`

#### Scenario: Type checking
- **WHEN** user calls `(cut 42 (make-sphere 10))`
- **THEN** the system SHALL signal a Janet type error

#### Scenario: Original shapes preserved
- **WHEN** user calls `(def a (make-box 10 10 10)) (def b (make-sphere 10)) (def result (cut a b))`
- **THEN** `a` and `b` SHALL remain valid shapes and `(visible? a)` SHALL return true

### Requirement: Common (intersect) two shapes

The system SHALL provide a function `common` that performs a boolean intersection operation using the OCCT BRepAlgoAPI_Common kernel.

The function SHALL accept two required positional arguments, both of which MUST be `rojcad/shape` abstract values.

The function SHALL return a new `rojcad/shape` abstract value representing the volume shared by both input shapes. The original operand shapes SHALL remain unchanged.

If the boolean operation fails (e.g., shapes do not intersect), the system SHALL signal a Janet error with a descriptive message.

#### Scenario: Common between overlapping shapes
- **WHEN** user calls `(common (make-sphere 10) (make-box 10 10 10))`
- **THEN** the system returns a shape representing the intersection of the sphere and box

#### Scenario: Non-overlapping shapes
- **WHEN** user calls `(common (make-sphere 10) (make-box 10 10 10 :center '(100 0 0)))`
- **THEN** the system SHALL signal a Janet error indicating the shapes do not intersect

#### Scenario: Common is commutative
- **WHEN** user calls `(def a (make-sphere 10)) (def b (make-box 10 10 10))`
- **THEN** `(common a b)` SHALL produce the same result as `(common b a)`
