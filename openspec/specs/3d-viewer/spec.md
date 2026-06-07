## Requirements

### Requirement: Viewer window opens automatically

The system SHALL open an interactive 3D viewer window on startup unless the `--headless` flag is provided.

#### Scenario: Default startup opens viewer
- **WHEN** the user runs `cargo run --release` without flags
- **THEN** a window titled "rojcad — 3D Viewer" appears alongside the terminal REPL

#### Scenario: Headless flag suppresses viewer
- **WHEN** the user runs `cargo run --release -- --headless`
- **THEN** no viewer window is created and the system behaves as a purely headless server

### Requirement: Camera controls

The viewer SHALL support orbit, pan, and zoom camera controls via mouse and keyboard.

#### Scenario: Orbit rotation
- **WHEN** the user left-drags in the viewport
- **THEN** the camera orbits around the scene origin

#### Scenario: Pan
- **WHEN** the user middle-drags in the viewport
- **THEN** the view pans in the drag direction

#### Scenario: Zoom
- **WHEN** the user right-drags or scrolls in the viewport
- **THEN** the view zooms in or out

#### Scenario: Perspective/orthographic toggle
- **WHEN** the user presses `P` or `O`
- **THEN** the projection toggles between perspective and orthographic

### Requirement: Mesh surface rendering

The viewer SHALL tessellate OCCT shapes via `shape.mesh()` and render triangle meshes with smooth shading.

#### Scenario: Shape appears as solid mesh
- **WHEN** a shape is created in the REPL via `(make-box 10 20 30)`
- **THEN** the viewer displays the box as a shaded solid mesh with per-vertex normals

#### Scenario: Multiple shapes rendered
- **WHEN** two or more shapes exist in the registry
- **THEN** the viewer renders all visible shapes simultaneously

### Requirement: Vertex buffer contains interleaved position and normal data

The GPU vertex buffer SHALL contain interleaved `position: vec3<f32>` + `normal: vec3<f32>` data (24 bytes per vertex) matching the `MeshVertex` layout used by the render pipeline. Positions and normals from the same mesh index MUST occupy adjacent memory in the buffer.

#### Scenario: Sphere mesh renders without geometric corruption
- **WHEN** a sphere is created via `(make-sphere 5.0)` and rendered in the viewer
- **THEN** the sphere surface appears smooth and continuous with no gaps, no triangles collapsing to the origin, and no ~180° void

#### Scenario: All tessellated shapes use correct vertex layout
- **WHEN** any shape (box, cylinder, boolean result) is tessellated and uploaded to the GPU
- **THEN** every vertex in the buffer contains both position and normal data, and the GPU pipeline reads exactly `sizeof(MeshVertex)` bytes per vertex

### Requirement: Edge wireframe overlay

The viewer SHALL render topological edges of each shape as smooth polylines overlaid on the mesh surface.

#### Scenario: Edges visible on shape
- **WHEN** a shape is displayed
- **THEN** its topological edges are drawn as thin dark lines on top of the mesh surface

#### Scenario: Back-edge toggle
- **WHEN** the user presses `X`
- **THEN** edges on the back faces are shown as dashed lines, and pressing `X` again hides them

### Requirement: Helper grid

The viewer SHALL display a planar grid on the XZ plane (Y = 0) and a world-space axis indicator.

#### Scenario: Grid is visible
- **WHEN** the viewer opens
- **THEN** a grid of major and minor lines is visible on the XZ plane, with the origin highlighted

#### Scenario: Axis indicator is visible
- **WHEN** the viewer opens
- **THEN** three colored axis lines (red = X, green = Y, blue = Z) with cone tips are visible at the origin

### Requirement: Shape visibility

The viewer SHALL respect the `visible` flag on each shape.

#### Scenario: Hidden shape removed from view
- **WHEN** the user calls `(hide s)` on a shape in the REPL
- **THEN** the shape disappears from the viewer

#### Scenario: Hidden shape restored
- **WHEN** the user calls `(show s)` on a previously hidden shape
- **THEN** the shape reappears in the viewer

### Requirement: Viewport close behavior

The viewer SHALL handle window close gracefully without terminating the server.

#### Scenario: Closing viewer window
- **WHEN** the user closes the viewer window
- **THEN** the REPL server continues running; the viewer can be reopened (future) or the user continues via REPL
