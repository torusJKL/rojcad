## Requirements

### Requirement: Translate a shape

The system SHALL provide a function `translate` that creates a translated copy of a shape using the OCCT BRepBuilderAPI_Transform kernel with `copy=true`.

The function SHALL accept one required positional `rojcad/shape` argument and three numeric arguments (dx, dy, dz) representing the translation offset.

The function SHALL return a new `rojcad/shape` abstract value. The original shape SHALL remain unchanged.

#### Scenario: Translate a box
- **WHEN** user calls `(translate (box 10 10 10) 5 0 0)`
- **THEN** the system returns a copy of the box shifted 5 units in the positive X direction

#### Scenario: Translate preserves original
- **WHEN** user calls `(def a (box 10 10 10)) (def b (translate a 5 0 0))`
- **THEN** `a` SHALL remain at the origin and `b` SHALL be at x=5

#### Scenario: Translate with keyword
- **WHEN** user calls `(translate (box 10 10 10) :t [1 2 3])`
- **THEN** the system returns the box shifted by (1, 2, 3)

#### Scenario: Type checking
- **WHEN** user calls `(translate 42 0 0 0)`
- **THEN** the system SHALL signal a Janet type error

### Requirement: Rotate a shape

The system SHALL provide a function `rotate` that creates a rotated copy of a shape using the OCCT BRepBuilderAPI_Transform kernel with `copy=true`.

The function SHALL accept one required positional `rojcad/shape` argument and an angle specified via the `:a` (degrees) or `:ar` (radians) keyword.

The function SHALL accept exactly one axis keyword: `:x`, `:y`, or `:z` for cardinal axes, or `:r [dx dy dz]` for a custom axis direction.

The function SHALL return a new `rojcad/shape` abstract value. The original shape SHALL remain unchanged.

#### Scenario: Rotate about Z axis
- **WHEN** user calls `(rotate (box 10 10 10) :a 45 :z)`
- **THEN** the system returns a copy of the box rotated 45 degrees about the Z axis

#### Scenario: Rotate about X axis in radians
- **WHEN** user calls `(rotate (box 10 10 10) :ar 1.5708 :x)`
- **THEN** the system returns a copy rotated approximately 90 degrees about the X axis

#### Scenario: Rotate about custom axis
- **WHEN** user calls `(rotate (box 10 10 10) :a 90 :r [1 1 0])`
- **THEN** the system returns a copy rotated 90 degrees about the (1, 1, 0) axis

#### Scenario: Rotate preserves original
- **WHEN** user calls `(def a (box 10 10 10)) (def b (rotate a :a 90 :z))`
- **THEN** `a` SHALL remain unrotated and `b` SHALL be rotated

#### Scenario: Missing axis keyword
- **WHEN** user calls `(rotate (box 10 10 10) :a 45)`
- **THEN** the system SHALL signal a Janet error indicating an axis is required

#### Scenario: Type checking
- **WHEN** user calls `(rotate 42 :a 45 :z)`
- **THEN** the system SHALL signal a Janet type error

### Requirement: Scale a shape

The system SHALL provide a function `scale` that creates a uniformly scaled copy of a shape using the OCCT BRepBuilderAPI_Transform kernel with `copy=true`.

The function SHALL accept one required positional `rojcad/shape` argument and one numeric argument (factor) representing the uniform scale factor.

The function SHALL accept an optional keyword argument `:o [x y z]` specifying the center point of the scaling operation. When omitted, the center defaults to (0, 0, 0).

The function SHALL return a new `rojcad/shape` abstract value. The original shape SHALL remain unchanged.

#### Scenario: Scale about origin
- **WHEN** user calls `(scale (box 10 10 10) 2.0)`
- **THEN** the system returns a 20×20×20 box scaled 2× about the origin

#### Scenario: Scale about custom point
- **WHEN** user calls `(scale (box 10 10 10) 2.0 :o [5 5 5])`
- **THEN** the system returns a box scaled 2× about the point (5, 5, 5)

#### Scenario: Scale with factor < 1
- **WHEN** user calls `(scale (box 10 10 10) 0.5)`
- **THEN** the system returns a 5×5×5 box

#### Scenario: Scale preserves original
- **WHEN** user calls `(def a (box 10 10 10)) (def b (scale a 2.0))`
- **THEN** `a` SHALL remain unchanged and `b` SHALL be twice the size

#### Scenario: Negative scale factor
- **WHEN** user calls `(scale (box 10 10 10) -1.0)`
- **THEN** the system returns a mirrored copy (reflection through origin)

#### Scenario: Type checking
- **WHEN** user calls `(scale 42 2.0)`
- **THEN** the system SHALL signal a Janet type error

### Requirement: Mirror a shape

The system SHALL provide a function `mirror` that creates a mirrored copy of a shape using the OCCT BRepBuilderAPI_Transform kernel with `copy=true`.

The function SHALL accept one required positional `rojcad/shape` argument and six numeric arguments: (ox, oy, oz) for a point on the mirror axis, and (dx, dy, dz) for the axis direction.

The function SHALL return a new `rojcad/shape` abstract value. The original shape SHALL remain unchanged.

#### Scenario: Mirror about X axis
- **WHEN** user calls `(mirror (box 10 10 10) 0 0 0 1 0 0)`
- **THEN** the system returns a copy of the box mirrored across the X axis through the origin

#### Scenario: Mirror about arbitrary axis
- **WHEN** user calls `(mirror (box 10 10 10) 5 0 0 0 1 0)`
- **THEN** the system returns a copy mirrored across a Y-directed axis through (5, 0, 0)

#### Scenario: Mirror preserves original
- **WHEN** user calls `(def a (box 10 10 10)) (def b (mirror a 0 0 0 0 0 1))`
- **THEN** `a` SHALL remain unchanged and `b` SHALL be mirrored

#### Scenario: Type checking
- **WHEN** user calls `(mirror 42 0 0 0 1 0 0)`
- **THEN** the system SHALL signal a Janet type error
