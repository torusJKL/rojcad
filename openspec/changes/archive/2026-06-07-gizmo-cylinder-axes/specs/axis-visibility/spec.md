## ADDED Requirements

### Requirement: Axis shafts always visible
The gizmo axis shafts SHALL be rendered as 3D geometry with non-zero thickness in all directions, so they are visible from any camera angle.

#### Scenario: View from arbitrary angle
- **WHEN** the camera orbits to any position around the gizmo
- **THEN** each of the three axis shafts (X red, Y green, Z blue) SHALL be visible as a colored rod

#### Scenario: View from alignment-critical angle
- **WHEN** the camera is positioned along any cardinal axis direction (e.g. looking exactly down +Z)
- **THEN** the two orthogonal axis shafts (e.g. X and Y) SHALL still be visible with their full color and thickness
