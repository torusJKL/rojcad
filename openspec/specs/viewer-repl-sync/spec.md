## Requirements

### Requirement: Shape registry

The system SHALL maintain a shared `ShapeRegistry` that both the REPL thread and viewer thread can access. The registry maps shape IDs to shape data including tessellated mesh and edge polylines.

#### Scenario: Shape registered after creation
- **WHEN** a shape is created via `(make-box 10 20 30)` in the REPL
- **THEN** the shape is registered in the `ShapeRegistry` with a unique ID and its `mesh()` tessellation is available

#### Scenario: Shape registry accessible from viewer
- **WHEN** the viewer thread renders a frame
- **THEN** it reads the current state of the `ShapeRegistry` to determine which shapes to render

### Requirement: Automatic viewer update

The viewer SHALL automatically update when shapes are created, modified, or have their visibility changed in the REPL.

#### Scenario: New shape appears in viewer
- **WHEN** a new shape is created in the REPL
- **THEN** the viewer displays the new shape within the next frame without user intervention

#### Scenario: Modified shape updates
- **WHEN** a boolean operation creates a new result shape (e.g., `(cut a b)`)
- **THEN** the viewer updates to show the new result and (optionally) remove the operands

#### Scenario: Visibility change reflected
- **WHEN** `(hide s)` or `(show s)` is called
- **THEN** the viewer hides or shows the shape within the next frame

### Requirement: Selection event propagation

Selection events from the viewer SHALL be propagated to the Janet REPL as callbacks.

#### Scenario: Selection triggers Janet callback
- **WHEN** a shape is selected in the viewer
- **THEN** a registered Janet callback is invoked with the selected shape's ID

### Requirement: Thread safety

The `ShapeRegistry` SHALL be thread-safe, allowing concurrent reads from the viewer thread and writes from the REPL thread.

#### Scenario: Concurrent read and write
- **WHEN** the viewer thread reads the registry while the REPL thread writes to it
- **THEN** the read returns a consistent snapshot (either before or after the write, not a partial update)
