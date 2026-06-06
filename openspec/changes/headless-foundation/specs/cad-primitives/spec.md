## ADDED Requirements

### Requirement: Create a box primitive

The system SHALL provide a function `make-box` that creates a rectangular solid (box) using the OCCT BRepPrimAPI_MakeBox kernel.

The function SHALL accept three required positional numeric arguments: width (X dimension), depth (Y dimension), height (Z dimension). The box SHALL have one corner at the origin (0, 0, 0) and extend into positive X, Y, Z.

The function SHALL accept an optional keyword argument `:center` containing a tuple of three numbers `'(cx cy cz)` that translates the box so its geometric center is at the given coordinates. When `:center` is provided, the box SHALL be centered at that point (not anchored at origin).

The function SHALL return a Janet abstract value of type `rojcad/shape`.

The box SHALL be created with default metadata: visible = true, color = none.

#### Scenario: Box at origin
- **WHEN** user calls `(make-box 10 20 30)`
- **THEN** the system creates a box with width=10, depth=20, height=30 with one corner at (0,0,0)

#### Scenario: Box centered at a point
- **WHEN** user calls `(make-box 10 20 30 :center '(5 10 0))`
- **THEN** the system creates a box with width=10, depth=20, height=30 centered at (5, 10, 0)

#### Scenario: Box returns a shape value
- **WHEN** user calls `(def b (make-box 10 10 10))`
- **THEN** the variable `b` is bound to a `rojcad/shape` abstract value

#### Scenario: Invalid dimensions
- **WHEN** user calls `(make-box 0 10 10)` or `(make-box -5 10 10)`
- **THEN** the system SHALL signal a Janet error with a descriptive message

### Requirement: Create a sphere primitive

The system SHALL provide a function `make-sphere` that creates a solid sphere using the OCCT BRepPrimAPI_MakeSphere kernel.

The function SHALL accept one required positional numeric argument: radius.

The sphere SHALL be centered at the origin (0, 0, 0) by default.

The function SHALL accept an optional keyword argument `:center` containing a tuple of three numbers `'(cx cy cz)` that repositions the sphere center.

The function SHALL return a Janet abstract value of type `rojcad/shape`.

#### Scenario: Sphere at origin
- **WHEN** user calls `(make-sphere 10)`
- **THEN** the system creates a sphere with radius=10 centered at (0,0,0)

#### Scenario: Sphere with center
- **WHEN** user calls `(make-sphere 10 :center '(1 2 3))`
- **THEN** the system creates a sphere with radius=10 centered at (1, 2, 3)

#### Scenario: Invalid radius
- **WHEN** user calls `(make-sphere 0)` or `(make-sphere -1)`
- **THEN** the system SHALL signal a Janet error with a descriptive message
