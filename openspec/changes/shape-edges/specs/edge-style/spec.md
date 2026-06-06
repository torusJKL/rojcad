## ADDED Requirements

### Requirement: Runtime edge thickness control

The system SHALL provide a Janet function `edge-thickness` to get or set edge line thickness in NDC units.

#### Scenario: Query current thickness
- **WHEN** the user calls `(edge-thickness)` with no arguments
- **THEN** the current thickness value is returned as a number

#### Scenario: Set thickness
- **WHEN** the user calls `(edge-thickness 0.008)`
- **THEN** the edge thickness is updated immediately and subsequent renders use the new value

#### Scenario: Default thickness
- **WHEN** the viewer first opens
- **THEN** the edge thickness is 0.004 (approximately 3 pixels at 1024 viewport width)

### Requirement: Runtime inactive edge color control

The system SHALL provide a Janet function `edge-color-inactive` to set the color of edges on non-selected shapes using RGB values in [0, 1].

#### Scenario: Set inactive edge color
- **WHEN** the user calls `(edge-color-inactive 0.8 0.8 0.8)`
- **THEN** inactive edges render in light grey

#### Scenario: Invalid color values
- **WHEN** the user calls `(edge-color-inactive 1.5 0 0)`
- **THEN** the red component is clamped to 1.0

### Requirement: Runtime active edge color control

The system SHALL provide a Janet function `edge-color-active` to set the color of edges on the selected shape using RGB values in [0, 1].

#### Scenario: Set active edge color
- **WHEN** the user calls `(edge-color-active 0.3 0.5 1.0)`
- **THEN** edges of the selected shape render in light blue

#### Scenario: Active color independent of inactive color
- **WHEN** the user changes inactive color to red and active color to green
- **THEN** non-selected shape edges are red and selected shape edges are green
