## ADDED Requirements

### Requirement: Gizmo viewport overlay

The system SHALL render an axis orientation gizmo in a dedicated viewport overlay at the top-right corner of the viewer window.

#### Scenario: Gizmo visible on startup
- **WHEN** the viewer window opens
- **THEN** a gizmo widget is visible in the top-right corner of the window

#### Scenario: Gizmo renders on top of scene
- **WHEN** a 3D scene is rendered
- **THEN** the gizmo appears on top of the scene content with proper alpha blending

### Requirement: Axis orientation display

The gizmo SHALL display three colored lines radiating from a central point, each ending in a sphere with the axis letter label (X, Y, Z) as a camera-facing dark-colored label.

#### Scenario: Axis lines and labels are colored correctly
- **WHEN** the gizmo is rendered
- **THEN** the X axis line and sphere are red (#F03752), the Y axis line and sphere are green (#76B316), and the Z axis line and sphere are blue (#2D89F0); the positive-direction spheres display "X", "Y", and "Z" labels respectively in dark color

#### Scenario: Gizmo rotates with the view
- **WHEN** the user orbits the camera
- **THEN** the gizmo rotates to match the main camera's orientation

#### Scenario: Labels always face the viewer
- **WHEN** the camera orbits around the gizmo
- **THEN** the axis letter labels always face the camera direction

### Requirement: No secondary spheres

The gizmo SHALL NOT display spheres on the negative axis directions.

#### Scenario: Only three spheres visible
- **WHEN** the gizmo is rendered
- **THEN** exactly three colored spheres are visible at the +X, +Y, and +Z axis endpoints

### Requirement: Keyboard view shortcuts

The system SHALL provide keyboard shortcuts to switch the camera to standard orthographic plane views with a 500ms ease-in-out animation.

#### Scenario: Ctrl+1 toggles front/back view
- **WHEN** the user presses Ctrl+1
- **THEN** the camera animates to the +Z (front) orthographic view; pressing Ctrl+1 again animates to the -Z (back) view

#### Scenario: Ctrl+7 toggles top/bottom view
- **WHEN** the user presses Ctrl+7
- **THEN** the camera animates to the +Y (top) orthographic view; pressing Ctrl+7 again animates to the -Y (bottom) view

#### Scenario: Ctrl+3 toggles left/right view
- **WHEN** the user presses Ctrl+3
- **THEN** the camera animates to the -X (left) orthographic view; pressing Ctrl+3 again animates to the +X (right) view

#### Scenario: Different shortcut resets toggle
- **WHEN** the user presses Ctrl+1 (front), then Ctrl+3 (left), then Ctrl+1 again
- **THEN** the camera goes to the +Z (front) view (not -Z) because a different shortcut was pressed in between

#### Scenario: Manual rotation cancels animation
- **WHEN** a camera animation is in progress and the user starts rotating the view manually
- **THEN** the animation stops immediately and the user regains control

### Requirement: Animation switches to orthographic

When a keyboard shortcut triggers a plane view, the system SHALL switch the main camera to orthographic projection on animation completion.

#### Scenario: Animation switches to orthographic
- **WHEN** the main camera is in perspective mode and the user presses a view shortcut
- **THEN** after the animation completes, the main camera switches to orthographic projection

### Requirement: Gizmo works in perspective and orthographic modes

The gizmo SHALL function identically regardless of the main camera's projection mode.

#### Scenario: Gizmo visible in perspective mode
- **WHEN** the main camera is in perspective mode
- **THEN** the gizmo is visible

#### Scenario: Gizmo visible in orthographic mode
- **WHEN** the main camera is in orthographic mode
- **THEN** the gizmo is visible

### Requirement: High-DPI support

The gizmo viewport SHALL scale proportionally to the window's DPI scale factor.

#### Scenario: Gizmo larger on high-DPI
- **WHEN** the viewer is on a high-DPI display (e.g., scale factor 2.0)
- **THEN** the gizmo viewport size is doubled compared to a standard-DPI display

### Requirement: Lines have visible thickness

The axis lines SHALL be rendered with a visible thickness of approximately 2-3 pixels at the base viewport.

#### Scenario: Lines visible at standard size
- **WHEN** the gizmo is rendered at the standard 200×200 pixel viewport
- **THEN** the axis lines are approximately 3 pixels wide
