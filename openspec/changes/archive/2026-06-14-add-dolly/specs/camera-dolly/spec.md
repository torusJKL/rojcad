## ADDED Requirements

### Requirement: Camera dolly forward/backward
The camera SHALL support dollying (translating the entire camera rig forward/backward along the view direction), distinct from zooming.

#### Scenario: Shift+scroll dolly in
- **WHEN** the user holds Shift and scrolls up
- **THEN** the camera rig SHALL translate forward along the view direction (target moves forward, radius unchanged)

#### Scenario: Shift+scroll dolly out
- **WHEN** the user holds Shift and scrolls down
- **THEN** the camera rig SHALL translate backward along the view direction (target moves backward, radius unchanged)

#### Scenario: Shift+RMB drag dolly in
- **WHEN** the user holds Shift and drags the right mouse button upward
- **THEN** the camera rig SHALL translate forward along the view direction

#### Scenario: Shift+RMB drag dolly out
- **WHEN** the user holds Shift and drags the right mouse button downward
- **THEN** the camera rig SHALL translate backward along the view direction

#### Scenario: Unmodified RMB still zooms
- **WHEN** the user drags the right mouse button without any modifier
- **THEN** the camera SHALL zoom (radius scales) as before — no behavior change

#### Scenario: Unmodified scroll still zooms
- **WHEN** the user scrolls without any modifier
- **THEN** the camera SHALL zoom (radius scales) as before — no behavior change

#### Scenario: Dolly preserves orbit center
- **WHEN** the user dollies forward then orbits
- **THEN** orbit SHALL pivot around the new target position (not the pre-dolly position)
