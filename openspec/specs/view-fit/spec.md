## Requirements

### Requirement: Fit camera to explicit shapes

`(view-fit shape & shapes ; :reset)` SHALL frame the 3D camera on the union bounding box of all provided shapes, using a 0.5s animated transition with ease-in-out interpolation. The current orbit angle SHALL be preserved by default. Use `:reset` to return to the default isometric angle (yaw=0, pitch=0.4). The union SHALL include all named shapes regardless of their current visibility state. Shapes that have never been shown (no mesh data) SHALL be silently excluded.

#### Scenario: Fit to single visible shape preserves angle
- **WHEN** a visible shape exists and `(view-fit my-shape)` is called
- **THEN** the camera SHALL animate to frame the shape's bounding box
- **AND** the yaw and pitch SHALL remain at their current values

#### Scenario: Fit to multiple shapes
- **WHEN** two shapes at different positions exist and `(view-fit box1 sphere2)` is called
- **THEN** the camera SHALL animate to frame the union bounding box encompassing both shapes

#### Scenario: Fit to mixed visible and hidden shapes
- **WHEN** one visible shape and one hidden shape exist and `(view-fit visible-shape hidden-shape)` is called
- **THEN** the camera SHALL frame the union bounding box of both shapes
- **AND** the hidden shape's mesh data SHALL be included in the computation

#### Scenario: Fit with :reset returns to default angle
- **WHEN** `(view-fit :reset my-shape)` is called after the user has orbited to a custom angle
- **THEN** the camera SHALL animate target and radius to frame the shape
- **AND** the yaw and pitch SHALL reset to (0, 0.4)

#### Scenario: Fit with no shapes panics
- **WHEN** `(view-fit)` is called with no arguments
- **THEN** an error SHALL be signaled: expected at least one shape

#### Scenario: Fit to shape that was never shown
- **WHEN** a shape created but never shown is passed to `(view-fit no-mesh-shape)`
- **THEN** the shape SHALL be silently skipped (no mesh data available)
- **AND** if no other shapes have mesh, no camera movement SHALL occur

### Requirement: Fit camera to all visible shapes

`(view-fit-all ; :hidden ; :reset)` SHALL frame the 3D camera on the union bounding box of shapes in the ShapeRegistry, using a 0.5s animated transition. By default only shapes with `visible=true` are included. When `:hidden` is specified, all registered shapes (visible and hidden) SHALL be included. The current orbit angle SHALL be preserved by default. Use `:reset` to return to the default isometric angle.

#### Scenario: Fit to all visible shapes preserves angle
- **WHEN** multiple visible shapes exist and `(view-fit-all)` is called
- **THEN** the camera SHALL animate to frame the union bounding box of all visible shapes
- **AND** the yaw and pitch SHALL remain at their current values

#### Scenario: Fit to all with hidden shapes excluded by default
- **WHEN** two shapes exist, one visible and one hidden, and `(view-fit-all)` is called
- **THEN** the camera SHALL frame only the visible shape's bounding box
- **AND** the hidden shape's bounds SHALL NOT be included

#### Scenario: Fit to all including hidden with :hidden
- **WHEN** two shapes exist, one visible and one hidden, and `(view-fit-all :hidden)` is called
- **THEN** the camera SHALL frame the union bounding box of both shapes
- **AND** the hidden shape's mesh data SHALL be included in the computation

#### Scenario: Fit-all with :reset returns to default angle
- **WHEN** shapes exist and `(view-fit-all :reset)` is called
- **THEN** the camera SHALL animate to frame all shapes
- **AND** the yaw and pitch SHALL reset to (0, 0.4)

#### Scenario: Fit-all with no shapes resets camera
- **WHEN** no shapes exist (or no shapes with mesh data) and `(view-fit-all)` is called
- **THEN** the camera SHALL reset to default position: target=origin, radius=50, yaw=0, pitch=0.4

#### Scenario: Fit-all with :hidden and :reset combined
- **WHEN** hidden and visible shapes exist and `(view-fit-all :hidden :reset)` is called
- **THEN** the camera SHALL frame the union of ALL shapes (visible + hidden)
- **AND** the yaw and pitch SHALL reset to (0, 0.4)

### Requirement: Camera animation over 0.5s with ease-in-out

Both `view-fit` and `view-fit-all` SHALL animate camera movement using smooth interpolation. The animation SHALL use the same `ease_in_out(t) = t²(3-2t)` function as the existing view-snaps system.

#### Scenario: Animated target transition
- **WHEN** a fit command triggers a camera movement
- **THEN** the camera target SHALL interpolate linearly from its current position to the bounding box center over 0.5 seconds
- **AND** the interpolation SHALL use ease-in-out easing

#### Scenario: Animated radius transition
- **WHEN** a fit command triggers a camera movement
- **THEN** the camera radius SHALL interpolate from its current value to the computed optimal distance over 0.5 seconds

### Requirement: Fit distance computation adapts to projection mode

The optimal camera distance SHALL be computed differently for perspective and orthographic modes.

#### Scenario: Perspective fit distance
- **WHEN** the camera is in perspective mode and a fit command is received
- **THEN** the target radius SHALL be computed as: `max(R/tan(fov/2), R/(tan(fov/2)*aspect)) * 1.3`

#### Scenario: Orthographic fit distance
- **WHEN** the camera is in orthographic mode and a fit command is received
- **THEN** the target radius SHALL be computed as: `max(2*R, 2*R/aspect) * 1.3`

### Requirement: REPL→Viewer communication via mpsc channel

The system SHALL provide a dedicated mpsc channel for sending commands from the Janet REPL thread to the viewer thread. The viewer SHALL poll this channel once per frame in its render loop.

#### Scenario: Fit command received and processed
- **WHEN** a `ReplToViewer::FitToBounds` message is sent on the channel
- **THEN** the viewer SHALL read it on the next frame
- **AND** SHALL call `ViewerState::fit_to_bounds()` with the enclosed parameters

### Requirement: Function documentation

Each new Janet function SHALL include a docstring that describes its usage, parameters, keyword options, and examples.

#### Scenario: view-fit docstring
- **WHEN** `(doc view-fit)` is called in the REPL
- **THEN** the output SHALL describe the `(view-fit shape & shapes ; :reset)` signature
- **AND** SHALL include at least one usage example
