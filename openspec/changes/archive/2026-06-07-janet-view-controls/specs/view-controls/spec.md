## ADDED Requirements

### Requirement: Hidden edges visibility

The `edge-hidden` family of functions SHALL control whether occluded (back-facing) edges are rendered as dashed lines. The initial default state SHALL be `false` (hidden).

#### Scenario: Query hidden edge state via (edge-hidden)

- **WHEN** `(edge-hidden)` is called with no arguments
- **THEN** the system SHALL return the current state: `true` if hidden edges are shown, `false` if hidden

#### Scenario: Set hidden edges visible via (edge-hidden true)

- **WHEN** `(edge-hidden true)` is called
- **THEN** hidden edges SHALL be rendered as dashed lines
- **AND** subsequent `(edge-hidden)` calls SHALL return `true`

#### Scenario: Set hidden edges hidden via (edge-hidden false)

- **WHEN** `(edge-hidden false)` is called
- **THEN** hidden edges SHALL NOT be rendered
- **AND** subsequent `(edge-hidden)` calls SHALL return `false`

#### Scenario: Toggle hidden edges via (edge-hidden-toggle)

- **WHEN** `(edge-hidden-toggle)` is called
- **THEN** the system SHALL flip the current hidden edge visibility state
- **AND** return the new state

#### Scenario: Query hidden edge state via (edge-hidden-show?)

- **WHEN** `(edge-hidden-show?)` is called
- **THEN** the system SHALL return `true` if hidden edges are currently shown, `false` otherwise

#### Scenario: X keyboard shortcut toggles hidden edges

- **WHEN** the `X` key is pressed in the viewer window
- **THEN** the system SHALL toggle hidden edge visibility identically to `(edge-hidden-toggle)`
- **AND** the new state SHALL be reflected by `(edge-hidden-show?)`

### Requirement: Projection mode control

The `projection-perspective` and `projection-toggle` functions SHALL control whether the camera uses perspective or orthographic projection. The initial default SHALL be perspective (`true`).

#### Scenario: Query projection mode via (projection-perspective)

- **WHEN** `(projection-perspective)` is called with no arguments
- **THEN** the system SHALL return `true` if perspective mode is active, `false` if orthographic

#### Scenario: Set perspective mode via (projection-perspective true)

- **WHEN** `(projection-perspective true)` is called
- **THEN** the camera SHALL switch to perspective projection
- **AND** subsequent `(projection-perspective)` calls SHALL return `true`

#### Scenario: Set orthographic mode via (projection-perspective false)

- **WHEN** `(projection-perspective false)` is called
- **THEN** the camera SHALL switch to orthographic projection
- **AND** subsequent `(projection-perspective)` calls SHALL return `false`

#### Scenario: Toggle projection via (projection-toggle)

- **WHEN** `(projection-toggle)` is called
- **THEN** the camera SHALL toggle between perspective and orthographic projection
- **AND** return the new state

#### Scenario: O/P keyboard shortcut toggles projection

- **WHEN** the `O` or `P` key is pressed in the viewer window
- **THEN** the system SHALL toggle projection mode identically to `(projection-toggle)`
- **AND** the new state SHALL be reflected by `(projection-perspective)`
