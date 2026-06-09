## ADDED Requirements

### Requirement: Screen-space panning
The camera SHALL pan in screen space: dragging the mouse left/right SHALL move the scene left/right on screen, and dragging up/down SHALL move the scene up/down on screen, at any camera pitch angle.

#### Scenario: Pan left at default pitch
- **WHEN** the camera pitch is 0.4 rad (default) and the user drags middle mouse button to the right by 100 pixels
- **THEN** the scene SHALL move right on screen (target translates along screen-right vector)

#### Scenario: Pan right at default pitch
- **WHEN** the camera pitch is 0.4 rad (default) and the user drags middle mouse button to the left by 100 pixels
- **THEN** the scene SHALL move left on screen (target translates along screen-left vector)

#### Scenario: Pan up at default pitch
- **WHEN** the camera pitch is 0.4 rad (default) and the user drags middle mouse button up by 100 pixels
- **THEN** the scene SHALL move up on screen (target translates along screen-up vector)

#### Scenario: Pan down at default pitch
- **WHEN** the camera pitch is 0.4 rad (default) and the user drags middle mouse button down by 100 pixels
- **THEN** the scene SHALL move down on screen (target translates along screen-down vector)

#### Scenario: Pan left at steep pitch
- **WHEN** the camera pitch is 1.0 rad (~57°) and the user drags middle mouse button to the right by 100 pixels
- **THEN** the scene SHALL move right on screen (target translates along screen-right vector, which at high pitch has a significant depth component)

#### Scenario: Pan axes are orthogonal at any pitch
- **WHEN** the camera has arbitrary pitch and yaw
- **THEN** the right and up pan vectors SHALL be orthogonal (dot product ≈ 0 within floating-point precision)

#### Scenario: Pan with Shift+left-drag also uses screen space
- **WHEN** the camera pitch is non-zero and the user holds Shift while left-dragging
- **THEN** panning SHALL behave identically to middle-mouse-drag (screen-space)
