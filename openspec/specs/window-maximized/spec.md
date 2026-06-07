## ADDED Requirements

### Requirement: Maximized by default

The viewer SHALL start maximized when no `--width` or `--height` CLI flags are provided. A maximized window fills the screen but retains its title bar and window decorations.

#### Scenario: Maximized on default start

- **WHEN** the application starts without `--width` or `--height` flags
- **THEN** the viewer window SHALL be maximized

### Requirement: CLI size flags disable maximized

Providing `--width` or `--height` (or both) SHALL disable maximized startup, resulting in a windowed mode at the specified (or default) dimensions.

#### Scenario: Width disables maximized

- **WHEN** the application starts with `--width 1280`
- **THEN** the viewer SHALL start in windowed mode (not maximized)

#### Scenario: Height disables maximized

- **WHEN** the application starts with `--height 720`
- **THEN** the viewer SHALL start in windowed mode (not maximized)

### Requirement: Runtime maximized toggle via Janet

The system SHALL expose `(window-maximized true)` and `(window-maximized false)` to enter or exit maximized state at runtime from the Janet REPL. The function SHALL accept exactly one boolean argument.

#### Scenario: Maximize

- **WHEN** a user calls `(window-maximized true)` from the Janet REPL
- **THEN** the viewer window SHALL be maximized

#### Scenario: Restore

- **WHEN** a user calls `(window-maximized false)` while the viewer is maximized
- **THEN** the viewer window SHALL restore to its previous windowed size

### Requirement: Runtime maximized state query

The system SHALL expose `(window-maximized?)` to query the current maximized state from the Janet REPL. The function SHALL return `true` if the viewer window is maximized, `false` otherwise.

#### Scenario: Query maximized state when maximized

- **WHEN** a user is in maximized mode and calls `(window-maximized?)`
- **THEN** the function SHALL return `true`

#### Scenario: Query maximized state when windowed

- **WHEN** a user is in windowed mode and calls `(window-maximized?)`
- **THEN** the function SHALL return `false`

#### Scenario: State updates after toggle

- **WHEN** a user calls `(window-maximized true)` then calls `(window-maximized?)`
- **THEN** the result SHALL be `true`
