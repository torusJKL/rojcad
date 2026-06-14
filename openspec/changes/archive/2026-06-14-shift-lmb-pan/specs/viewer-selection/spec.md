## MODIFIED Requirements

### Requirement: Selection fires on mouse release, not press

The 3D viewer SHALL trigger selection changes on mouse button release, not press. A drag threshold SHALL distinguish clicks from drags to prevent accidental selection during camera orbit or pan.

#### Scenario: Click selects on release
- **WHEN** user presses and releases the left mouse button without significant movement
- **THEN** the selection change is applied on release

#### Scenario: Drag does not trigger selection
- **WHEN** user presses the left mouse button, moves the cursor more than 3px, then releases
- **THEN** no selection change occurs
- **AND** the camera orbits during the drag (without Shift) or pans during the drag (with Shift)
