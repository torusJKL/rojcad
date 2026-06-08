## MODIFIED Requirements

### Requirement: Create a box primitive

The system SHALL provide a function `box` that creates a rectangular solid using the OCCT BRepPrimAPI_MakeBox kernel.

The function SHALL accept three positional numeric arguments: width (X), depth (Y), height (Z), with one corner at origin and extending into positive X, Y, Z. A single numeric argument SHALL create a cube.

The function SHALL accept optional keyword arguments: `:w` (width), `:d` (depth), `:h` (height), `:c` (center as `[cx cy cz]`), `:pl` (low corner as `[x y z]`), `:ph` (high corner as `[x y z]`).

When `:c` is provided, the box SHALL be centered at that point. When `:pl` and `:ph` are provided, the box SHALL extend between those two opposite corners.

The function SHALL return a `rojcad/shape` abstract value.

#### Scenario: Box at origin
- **WHEN** user calls `(box 10 20 30)`
- **THEN** the system creates a box with width=10, depth=20, height=30 with one corner at (0,0,0)

#### Scenario: Box centered
- **WHEN** user calls `(box 10 20 30 :c [5 10 0])`
- **THEN** the system creates a box centered at (5, 10, 0)

#### Scenario: Cube via single argument
- **WHEN** user calls `(box 5)`
- **THEN** the system creates a 5×5×5 cube with one corner at (0,0,0)

#### Scenario: Cube centered via single argument
- **WHEN** user calls `(box 5 :c [0 0 0])`
- **THEN** the system creates a 5×5×5 cube centered at origin

#### Scenario: Box from corners
- **WHEN** user calls `(box :pl [0 0 0] :ph [10 20 30])`
- **THEN** the system creates a box spanning from (0,0,0) to (10,20,30)

#### Scenario: Box with keyword args
- **WHEN** user calls `(box :w 10 :d 20 :h 30)`
- **THEN** the system creates a box with width=10, depth=20, height=30

#### Scenario: Invalid dimensions
- **WHEN** user calls `(box 0 10 10)`
- **THEN** the system SHALL signal a Janet error

### Requirement: Create a sphere primitive

The system SHALL provide a function `sphere` that creates a solid sphere using the OCCT BRepPrimAPI_MakeSphere kernel.

The function SHALL accept one positional numeric argument: radius. The sphere SHALL be centered at the origin by default.

The function SHALL accept optional keyword arguments: `:r` (radius), `:c` (center as `[x y z]`), `:a` (angle in radians, partial sphere).

The function SHALL return a `rojcad/shape` abstract value.

#### Scenario: Sphere at origin
- **WHEN** user calls `(sphere 10)`
- **THEN** the system creates a sphere with radius=10 centered at (0,0,0)

#### Scenario: Sphere with center
- **WHEN** user calls `(sphere 10 :c [1 2 3])`
- **THEN** the system creates a sphere centered at (1, 2, 3)

#### Scenario: Partial sphere
- **WHEN** user calls `(sphere 10 :a 3.14159)`
- **THEN** the system creates a hemisphere (half sphere)

#### Scenario: Invalid radius
- **WHEN** user calls `(sphere -1)`
- **THEN** the system SHALL signal a Janet error

## ADDED Requirements

### Requirement: Create a cylinder primitive

The system SHALL provide a function `cylinder` that creates a solid cylinder using the OCCT BRepPrimAPI_MakeCylinder kernel.

The function SHALL accept two positional numeric arguments: radius and height. The cylinder SHALL extend along the Z axis with its base at Z=0.

The function SHALL accept optional keyword arguments: `:r` (radius), `:h` (height), `:c` (center as `[x y z]`), `:dir` (direction as `[dx dy dz]`), `:fp` (from-point `[x y z]`), `:tp` (to-point `[x y z]`).

The function SHALL return a `rojcad/shape` abstract value.

#### Scenario: Cylinder at origin
- **WHEN** user calls `(cylinder 5 10)`
- **THEN** the system creates a cylinder with radius=5, height=10 along Z axis

#### Scenario: Cylinder centered
- **WHEN** user calls `(cylinder 5 10 :c [0 0 5])`
- **THEN** the system creates a cylinder centered at (0, 0, 5)

#### Scenario: Cylinder from points
- **WHEN** user calls `(cylinder :fp [0 0 0] :tp [0 0 10] :r 5)`
- **THEN** the system creates a cylinder between (0,0,0) and (0,0,10) with radius=5

#### Scenario: Invalid cylinder dimensions
- **WHEN** user calls `(cylinder 0 10)`
- **THEN** the system SHALL signal a Janet error

### Requirement: Create a cone primitive

The system SHALL provide a function `cone` that creates a solid cone using the OCCT BRepPrimAPI_MakeCone kernel.

The function SHALL accept two or three positional numeric arguments: bottom_radius, height (full cone); or bottom_radius, top_radius, height (truncated cone).

The function SHALL accept optional keyword arguments: `:br` (bottom radius), `:tr` (top radius), `:h` (height), `:c` (center as `[x y z]`), `:a` (angle in radians, partial cone).

The function SHALL return a `rojcad/shape` abstract value.

#### Scenario: Full cone
- **WHEN** user calls `(cone 5 10)`
- **THEN** the system creates a cone with bottom_radius=5, height=10, top_radius=0

#### Scenario: Truncated cone
- **WHEN** user calls `(cone 5 3 10)`
- **THEN** the system creates a truncated cone with bottom_radius=5, top_radius=3, height=10

#### Scenario: Partial cone
- **WHEN** user calls `(cone 5 10 :a 3.14159)`
- **THEN** the system creates a half-cone

#### Scenario: Invalid cone dimensions
- **WHEN** user calls `(cone -1 10)`
- **THEN** the system SHALL signal a Janet error

### Requirement: Create a torus primitive

The system SHALL provide a function `torus` that creates a solid torus using the OCCT BRepPrimAPI_MakeTorus kernel.

The function SHALL accept two positional numeric arguments: ring_radius (major) and tube_radius (minor).

The function SHALL accept optional keyword arguments: `:rr` (ring radius), `:tr` (tube radius), `:c` (center as `[x y z]`), `:a` (rotation angle in radians, partial torus), `:as` (start angle), `:ae` (end angle), `:dir` (axis direction as `[dx dy dz]`).

The function SHALL return a `rojcad/shape` abstract value.

#### Scenario: Full torus
- **WHEN** user calls `(torus 20 10)`
- **THEN** the system creates a torus with ring_radius=20, tube_radius=10

#### Scenario: Partial torus
- **WHEN** user calls `(torus 20 10 :a 3.14159)`
- **THEN** the system creates a half-torus

#### Scenario: Torus with angle range
- **WHEN** user calls `(torus :rr 20 :tr 10 :as 0 :ae 3.14159)`
- **THEN** the system creates a torus from angle 0 to pi

#### Scenario: Invalid torus dimensions
- **WHEN** user calls `(torus 20 0)`
- **THEN** the system SHALL signal a Janet error

## RENAMED Requirements

### Requirement: Rename make-box to box
- **FROM**: `make-box`
- **TO**: `box`

### Requirement: Rename make-sphere to sphere
- **FROM**: `make-sphere`
- **TO**: `sphere`
