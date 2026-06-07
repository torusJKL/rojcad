## ADDED Requirements

### Requirement: Generic view-angle function

The system SHALL provide a Janet-callable function `view-angle` that sets the camera to arbitrary yaw and pitch angles. The function SHALL accept two required numeric arguments (yaw, pitch in radians) and one optional numeric argument (distance from target). If distance is omitted, the current camera radius SHALL be preserved. The camera SHALL animate smoothly to the target angles over approximately 0.5 seconds using ease-in-out interpolation.

#### Scenario: Set camera to arbitrary angle
- **WHEN** `(view-angle 0 1.57)` is evaluated
- **THEN** the camera SHALL animate to yaw=0, pitch=1.57 rad over 0.5 seconds

#### Scenario: Set camera angle with zoom
- **WHEN** `(view-angle 0.785 0.615 150)` is evaluated
- **THEN** the camera SHALL animate to yaw=0.785 rad, pitch=0.615 rad, radius=150 over 0.5 seconds

#### Scenario: Distance omitted preserves current radius
- **WHEN** `(view-angle 0 0)` is evaluated
- **THEN** the camera SHALL animate to yaw=0, pitch=0 but the current camera radius SHALL NOT change

#### Scenario: Error on fewer than 2 arguments
- **WHEN** `(view-angle)` or `(view-angle 1)` is evaluated
- **THEN** the system SHALL signal a Janet error indicating wrong argument count

### Requirement: Named view presets

The system SHALL provide seven named view-preset functions, each wrapping `view-angle` with preset yaw/pitch values. Each function SHALL accept an optional distance argument. The presets SHALL be:

| Function | Yaw | Pitch | Looks from |
|----------|-----|-------|------------|
| `view-front` | π/2 | 0 | +Z axis |
| `view-back` | -π/2 | 0 | -Z axis |
| `view-right` | 0 | 0 | +X axis |
| `view-left` | π | 0 | -X axis |
| `view-top` | 0 | π/2 | +Y axis |
| `view-bottom` | 0 | -π/2 | -Y axis |
| `view-iso` | π/4 | asin(1/√3) ≈ 0.615 | (1,1,1) direction |

#### Scenario: Front view
- **WHEN** `(view-front)` is evaluated
- **THEN** the camera SHALL animate to yaw=π/2, pitch=0

#### Scenario: Front view with distance
- **WHEN** `(view-front 200)` is evaluated
- **THEN** the camera SHALL animate to yaw=π/2, pitch=0, radius=200

#### Scenario: Isometric view
- **WHEN** `(view-iso)` is evaluated
- **THEN** the camera SHALL animate to yaw=π/4, pitch=asin(1/√3) ≈ 0.615 rad

#### Scenario: Top view
- **WHEN** `(view-top)` is evaluated
- **THEN** the camera SHALL animate to yaw=0, pitch=π/2

#### Scenario: All named presets accept optional distance
- **WHEN** `(view-front 300)` or `(view-back 300)` or `(view-left 300)` or `(view-right 300)` or `(view-top 300)` or `(view-bottom 300)` or `(view-iso 300)` is evaluated
- **THEN** each SHALL animate to its preset yaw/pitch with the camera radius set to 300

### Requirement: Docstrings

Each named preset function and `view-angle` SHALL have a docstring that documents the yaw and pitch values achieved. Named preset docstrings SHALL include the yaw/pitch in the description. `view-angle` SHALL document its argument ordering.

#### Scenario: view-front docstring mentions yaw and pitch
- **WHEN** `(doc view-front)` is evaluated
- **THEN** the output SHALL contain "Yaw=π/2" and "Pitch=0"

#### Scenario: view-iso docstring mentions isometric angle
- **WHEN** `(doc view-iso)` is evaluated
- **THEN** the output SHALL contain "Yaw=π/4" and mention the asin(1/√3) pitch

### Requirement: Metadata for discoverability

Each named preset function and `view-angle` SHALL have `:source` set to `"rojcad"` and `:category` set to `"view"` for discoverability via `cad-fns` filtering.

#### Scenario: Functions have category metadata
- **WHEN** the system queries functions with category "view"
- **THEN** all 8 functions (`view-angle` + 7 named presets) SHALL appear in the results

### Requirement: Projection mode preserved

View angle animations SHALL NOT change the camera projection mode (perspective/orthographic). The current projection mode SHALL be preserved after animation completes.

#### Scenario: Perspective mode preserved after view change
- **WHEN** `(projection-perspective true)` then `(view-front)` is evaluated
- **THEN** after the animation completes, the camera SHALL remain in perspective mode

### Requirement: Animation cancellation

A new view-angle command SHALL cancel any in-progress camera animation (whether from a prior view-angle, view-fit, or keyboard shortcut).

#### Scenario: New view command cancels previous animation
- **WHEN** `(view-front)` is evaluated, then immediately `(view-back)` is evaluated
- **THEN** the camera SHALL animate directly to the back view, skipping the front view

### Requirement: No math.h dependency in C

The `view-angle` C JANET_FN SHALL NOT require `<math.h>`. All angle constants SHALL be computed in Janet using `math/pi`, `math/sqrt`, and `math/asin`.

#### Scenario: Angle computed in Janet
- **WHEN** `(view-iso)` is evaluated
- **THEN** the yaw=π/4 and pitch=asin(1/√3) SHALL be computed in Janet and passed as raw doubles through the C bridge
