## ADDED Requirements

### Requirement: Stats overlay renders real-time viewer information

A text-based overlay SHALL render in a draggable egui floating window, displaying current camera state, scene statistics, geometry metrics, toggle states, and rendering performance.

#### Scenario: Overlay shows camera yaw and pitch in degrees
- **WHEN** the user orbits the camera with left-click drag
- **THEN** the overlay updates yaw and pitch values in degrees (0-360°, -90° to +90°)

#### Scenario: Overlay shows zoom radius
- **WHEN** the user zooms with scroll wheel or right-click drag
- **THEN** the overlay updates the zoom/radius value

#### Scenario: Overlay shows projection mode
- **WHEN** the camera projection mode changes (P or O key)
- **THEN** the overlay shows "Perspective" or "Orthographic"

#### Scenario: Overlay shows a human-readable view name
- **WHEN** the camera angles match a known preset (Front, Back, Top, Bottom, Right, Left) within tolerance
- **THEN** the overlay displays the preset name
- **AND** when the camera is orbited away from any preset, the overlay displays "Custom"

#### Scenario: Overlay shows shape counts
- **WHEN** shapes exist in the registry
- **THEN** the overlay shows total, visible, and hidden counts
- **AND** counts update immediately when shapes are shown/hidden/created/removed

#### Scenario: Overlay shows selected shape info
- **WHEN** a shape is selected via click
- **THEN** the overlay shows the shape ID

#### Scenario: Overlay shows triangle and vertex counts
- **WHEN** visible shapes with meshes exist
- **THEN** the overlay shows the sum of their triangle counts (indices / 3) and vertex counts

#### Scenario: Overlay shows FPS and frame time
- **WHEN** the viewer is rendering frames
- **THEN** the overlay shows a smoothed FPS counter and per-frame time in milliseconds

#### Scenario: Overlay shows toggle states
- **WHEN** back-edge visibility is toggled (X key)
- **THEN** the overlay back-edges indicator updates to ON or OFF
- **WHEN** the overlay itself is toggled
- **THEN** the overlay visibility indicator reflects the current state

#### Scenario: Stats window is draggable
- **WHEN** the user drags the stats window title bar
- **THEN** the window follows the mouse
- **AND** the gzmo/camera controls do NOT activate during the drag

### Requirement: Overlay can be toggled via keyboard shortcut

The overlay SHALL toggle on/off with the `Ctrl + Shift + Alt + S` keyboard combination.

#### Scenario: Keyboard shortcut toggles overlay visibility
- **WHEN** the overlay is visible and the user presses Ctrl+Shift+Alt+S
- **THEN** the overlay becomes hidden
- **WHEN** the overlay is hidden and the user presses Ctrl+Shift+Alt+S
- **THEN** the overlay becomes visible

### Requirement: Overlay can be toggled via Janet function

A `stats-overlay` Janet function SHALL toggle, query, and set overlay visibility, following the same pattern as `edge-hidden` and `projection-perspective`.

#### Scenario: stats-overlay with no args queries state
- **WHEN** `(stats-overlay)` is evaluated with no arguments
- **THEN** it returns true if the overlay is visible, false if hidden

#### Scenario: stats-overlay with boolean arg sets state
- **WHEN** `(stats-overlay true)` is evaluated
- **THEN** the overlay becomes visible
- **WHEN** `(stats-overlay false)` is evaluated
- **THEN** the overlay becomes hidden

### Requirement: Overlay renders via egui

The overlay SHALL use egui for all text rendering, layout, and window management. egui-wgpu SHALL render the overlay in a separate render pass after the scene and gzmo passes.

#### Scenario: egui is initialized at viewer startup
- **WHEN** the viewer initializes
- **THEN** an egui context, egui-winit state, and egui-wgpu renderer are created

#### Scenario: egui events are processed before camera events
- **WHEN** a mouse or keyboard event reaches the viewer
- **THEN** egui processes it first via `egui_winit::State::on_window_event()`
- **AND** if egui consumed the event, camera controls are skipped for that event

#### Scenario: egui overlay renders after scene and gzmo
- **WHEN** the frame is rendered
- **THEN** the egui overlay is rendered after the gzmo overlay pass
- **AND** the egui pass uses `LoadOp::Load` with no depth/stencil attachment
