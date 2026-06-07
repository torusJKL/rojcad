## ADDED Requirements

### Requirement: CLI size flags imply windowed mode (not fullscreen)

Providing `--width` or `--height` (or both) SHALL result in a windowed mode at the specified dimensions, without any fullscreen or maximized state.

#### Scenario: Width implies windowed

- **WHEN** the application starts with `--width 1280`
- **THEN** the viewer SHALL start in windowed mode (not fullscreen)

#### Scenario: Height implies windowed

- **WHEN** the application starts with `--height 720`
- **THEN** the viewer SHALL start in windowed mode (not fullscreen)

### Requirement: Runtime fullscreen toggle via Janet

The system SHALL expose `(window-fullscreen true)` and `(window-fullscreen false)` to enter or exit fullscreen mode at runtime from the Janet REPL. The function SHALL accept exactly one boolean argument.

#### Scenario: Enter fullscreen

- **WHEN** a user calls `(window-fullscreen true)` from the Janet REPL
- **THEN** the viewer window SHALL enter borderless fullscreen on the current monitor

#### Scenario: Exit fullscreen

- **WHEN** a user calls `(window-fullscreen false)` while the viewer is in fullscreen
- **THEN** the viewer window SHALL exit fullscreen and restore to its previous windowed size

### Requirement: Runtime fullscreen state query

The system SHALL expose `(window-fullscreen?)` to query the current fullscreen state from the Janet REPL. The function SHALL return `true` if the viewer is in fullscreen, `false` otherwise.

#### Scenario: Query fullscreen state when in fullscreen

- **WHEN** a user is in fullscreen mode and calls `(window-fullscreen?)`
- **THEN** the function SHALL return `true`

#### Scenario: Query fullscreen state when windowed

- **WHEN** a user is in windowed mode and calls `(window-fullscreen?)`
- **THEN** the function SHALL return `false`

#### Scenario: State updates after toggle

- **WHEN** a user calls `(window-fullscreen true)` then calls `(window-fullscreen?)`
- **THEN** the result SHALL be `true`
