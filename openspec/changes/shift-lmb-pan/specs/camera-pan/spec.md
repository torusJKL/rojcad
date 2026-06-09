## ADDED Requirements

### Requirement: Camera pans via Shift+LMB drag

The camera SHALL pan when the user holds Shift and drags with the left mouse button. The pan speed and direction SHALL match the existing MMB pan behavior (proportional to drag distance and camera radius).

#### Scenario: Shift+LMB drag pans the view

- **WHEN** the user holds Shift, presses the left mouse button, and drags the mouse
- **THEN** the camera pans in the direction of the drag
- **AND** the existing click-vs-drag threshold (3px) SHALL NOT suppress panning during drag

#### Scenario: Shift+LMB drag does not trigger selection

- **WHEN** the user holds Shift, presses the left mouse button, drags the cursor beyond the click threshold (3px), then releases
- **THEN** no selection change occurs
- **AND** the camera pans during the drag

#### Scenario: Shift+LMB click (no drag) selects additively

- **WHEN** the user holds Shift and clicks the left mouse button without moving the cursor beyond the click threshold (3px)
- **THEN** the shape under the cursor is added to the selection set
- **AND** the camera does not pan

### Requirement: MMB drag remains as alternative pan method

The middle mouse button SHALL continue to pan the camera, preserving backward compatibility for users who prefer MMB panning.

#### Scenario: MMB drag pans the view

- **WHEN** the user presses the middle mouse button and drags the mouse
- **THEN** the camera pans in the direction of the drag
